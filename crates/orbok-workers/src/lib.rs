//! # orbok-workers
//!
//! Synchronous pipeline workers for M5/M6: pull queued jobs from the
//! catalog and execute them in dependency order.
//!
//! **Worker chain (per file):**
//! ```text
//! [Scan queues Extract]
//!   → ExtractionWorker  (extract + cache + record)
//!   → ChunkAndIndexWorker (chunk + FTS index + chunk_locations)
//! ```
//!
//! Failure isolation: one file's failure never stops the whole run
//! (RFC-004 §16, RFC-005 §13). Workers update the relevant catalog
//! records with the error category.

mod extract;
mod chunk_and_index;

#[cfg(test)]
mod tests;

pub use extract::ExtractionWorker;
pub use chunk_and_index::ChunkAndIndexWorker;

use orbok_core::OrbokResult;
use orbok_db::Catalog;
use orbok_core::{JobStatus, JobType};
use orbok_db::repo::IndexJobRepository;
use tracing::warn;

/// Run all queued jobs until the queue is empty or `limit` jobs have
/// been processed. Returns the number of jobs that succeeded.
pub fn run_pending(
    catalog: &Catalog,
    extract_worker: &ExtractionWorker<'_>,
    chunk_worker: &ChunkAndIndexWorker<'_>,
    limit: u32,
) -> OrbokResult<u64> {
    let jobs = IndexJobRepository::new(catalog);
    let mut succeeded = 0u64;
    let mut processed = 0u32;

    while processed < limit {
        let batch = jobs.list_queued(1)?;
        if batch.is_empty() {
            break;
        }
        let job = &batch[0];
        jobs.set_status(&job.job_id, JobStatus::Running)?;
        let result = match job.job_type {
            JobType::Extract => {
                if let Some(file_id) = &job.file_id {
                    extract_worker.run(file_id)
                } else {
                    Ok(())
                }
            }
            JobType::Chunk | JobType::KeywordIndex => {
                if let Some(file_id) = &job.file_id {
                    chunk_worker.run(file_id)
                } else {
                    Ok(())
                }
            }
            _ => Ok(()), // Other job types are no-ops in v0.2.
        };
        match result {
            Ok(()) => {
                jobs.set_status(&job.job_id, JobStatus::Succeeded)?;
                succeeded += 1;
            }
            Err(e) => {
                warn!(job = job.job_id.as_str(), error = %e, "job failed");
                jobs.set_status(&job.job_id, JobStatus::Failed)?;
            }
        }
        processed += 1;
    }
    Ok(succeeded)
}
