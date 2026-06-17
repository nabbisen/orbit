//! Index job queue repository (RFC-002 §7.9, RFC-004 §13).

use crate::catalog::{Catalog, db_err};
use orbok_core::{FileId, JobId, JobStatus, JobType, OrbokResult, SourceId, now_iso8601};
use rusqlite::params;

/// A queued or running index job.
#[derive(Debug, Clone)]
pub struct JobRecord {
    pub job_id: JobId,
    pub source_id: Option<SourceId>,
    pub file_id: Option<FileId>,
    pub job_type: JobType,
    pub status: JobStatus,
}

pub struct IndexJobRepository<'a> {
    catalog: &'a Catalog,
}

impl<'a> IndexJobRepository<'a> {
    pub fn new(catalog: &'a Catalog) -> Self {
        Self { catalog }
    }

    /// Enqueue a job (scanner queues `extract` for new/stale files,
    /// RFC-004 §13).
    pub fn enqueue(
        &self,
        job_type: JobType,
        source_id: Option<&SourceId>,
        file_id: Option<&FileId>,
    ) -> OrbokResult<JobId> {
        let id = JobId::generate();
        let now = now_iso8601();
        let conn = self.catalog.lock();
        conn.execute(
            "INSERT INTO index_jobs (job_id, source_id, file_id, job_type, status, \
             created_at, updated_at) VALUES (?1,?2,?3,?4,'queued',?5,?5)",
            params![
                id.as_str(),
                source_id.map(|s| s.as_str()),
                file_id.map(|f| f.as_str()),
                job_type.as_str(),
                now,
            ],
        )
        .map_err(db_err)?;
        Ok(id)
    }

    /// Move a job to a new status, recording start/completion times.
    pub fn set_status(&self, id: &JobId, status: JobStatus) -> OrbokResult<()> {
        let now = now_iso8601();
        let (started, completed) = match status {
            JobStatus::Running => (Some(now.clone()), None),
            JobStatus::Succeeded | JobStatus::Failed | JobStatus::Canceled => {
                (None, Some(now.clone()))
            }
            _ => (None, None),
        };
        let conn = self.catalog.lock();
        conn.execute(
            "UPDATE index_jobs SET status = ?2, updated_at = ?3, \
             started_at = COALESCE(?4, started_at), \
             completed_at = COALESCE(?5, completed_at) WHERE job_id = ?1",
            params![id.as_str(), status.as_str(), now, started, completed],
        )
        .map_err(db_err)?;
        Ok(())
    }

    /// Queued jobs in priority/FIFO order.
    pub fn list_queued(&self, limit: u32) -> OrbokResult<Vec<JobRecord>> {
        let conn = self.catalog.lock();
        let mut stmt = conn
            .prepare(
                "SELECT job_id, source_id, file_id, job_type, status FROM index_jobs \
                 WHERE status = 'queued' ORDER BY priority DESC, created_at LIMIT ?1",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map(params![limit], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(db_err)?;
        let mut out = Vec::new();
        for row in rows {
            let (id, src, file, jt, st) = row.map_err(db_err)?;
            out.push(JobRecord {
                job_id: JobId::from_string(id),
                source_id: src.map(SourceId::from_string),
                file_id: file.map(FileId::from_string),
                job_type: JobType::parse(&jt)?,
                status: JobStatus::parse(&st)?,
            });
        }
        Ok(out)
    }

    /// Count of jobs per status (Indexing view summary cards).
    pub fn count_by_status(&self) -> OrbokResult<Vec<(JobStatus, u64)>> {
        let conn = self.catalog.lock();
        let mut stmt = conn
            .prepare("SELECT status, COUNT(*) FROM index_jobs GROUP BY status")
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(db_err)?;
        let mut out = Vec::new();
        for row in rows {
            let (status, count) = row.map_err(db_err)?;
            out.push((JobStatus::parse(&status)?, count as u64));
        }
        Ok(out)
    }
}
