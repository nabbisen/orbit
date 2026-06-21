//! Scheduler resource limits and configuration (RFC-036 §9, §10.3).

/// Per-queue worker concurrency limits (RFC-036 §9.1).
///
/// Defaults are intentionally conservative: one calm worker per queue
/// is better than a fast app that feels hostile on a low-powered machine.
#[derive(Debug, Clone)]
pub struct SchedulerLimits {
    pub scan_workers: usize,
    pub extract_workers: usize,
    pub chunk_workers: usize,
    pub keyword_workers: usize,
    pub embedding_workers: usize,
    pub maintenance_workers: usize,
}

impl Default for SchedulerLimits {
    fn default() -> Self {
        Self {
            scan_workers: 1,
            extract_workers: 1,
            chunk_workers: 1,
            keyword_workers: 1,
            embedding_workers: 1,
            maintenance_workers: 1,
        }
    }
}

/// Per-queue depth caps (RFC-036 §10.3).
///
/// When a queue hits its cap, upstream production pauses (backpressure).
/// These are starting points; they will be tuned in RFC-037 benchmarks.
#[derive(Debug, Clone)]
pub struct QueueCapacity {
    pub scan_queue_max: usize,
    pub extract_queue_max: usize,
    pub chunk_queue_max: usize,
    pub keyword_queue_max: usize,
    pub embedding_queue_max: usize,
    pub maintenance_queue_max: usize,
}

impl Default for QueueCapacity {
    fn default() -> Self {
        Self {
            scan_queue_max: 10_000,
            extract_queue_max: 1_000,
            chunk_queue_max: 1_000,
            keyword_queue_max: 1_000,
            embedding_queue_max: 2_000,
            maintenance_queue_max: 200,
        }
    }
}

/// Maximum retry attempts before a job is marked permanently failed.
pub const MAX_JOB_ATTEMPTS: u32 = 3;

/// Complete scheduler configuration.
#[derive(Debug, Clone, Default)]
pub struct SchedulerConfig {
    pub limits: SchedulerLimits,
    pub capacity: QueueCapacity,
}
