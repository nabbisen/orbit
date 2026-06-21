//! Resource-aware indexing scheduler (RFC-036).
//!
//! Public surface:
//!
//! ```text
//! Scheduler        — dispatch engine
//! SchedulerConfig  — limits + queue capacities
//! SchedulerLimits  — per-queue worker counts
//! QueueCapacity    — per-queue depth caps
//! WorkPriority     — job priority levels
//! JobKind          — job type labels
//! JobState         — in-memory job state
//! IndexJob         — a single scheduler job
//! ResourceMode     — Normal / UserActive / LowImpact / Paused
//! SchedulerEvent   — UI event channel
//! QueueKind        — which queue for backpressure events
//! ```

pub mod dispatch;
pub mod job;
pub mod limits;
pub mod queue;

pub use dispatch::Scheduler;
pub use job::{IndexJob, JobKind, JobState, QueueKind, ResourceMode, SchedulerEvent, WorkPriority};
pub use limits::{MAX_JOB_ATTEMPTS, QueueCapacity, SchedulerConfig, SchedulerLimits};
pub use queue::{BoundedQueue, QueueSet};
