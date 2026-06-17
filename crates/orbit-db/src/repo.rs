//! Repository layer (RFC-002 §8). Each repository owns the SQL for one
//! table family and exposes application-level types.

pub mod cleanup;
pub mod events;
pub mod files;
pub mod jobs;
pub mod settings;
pub mod sources;
pub mod storage;

pub use cleanup::CleanupExecutor;
pub use events::{EventRepository, Severity};
pub use files::{FileRepository, FileRecord, NewFile, ObservedMetadata};
pub use jobs::{IndexJobRepository, JobRecord};
pub use settings::SettingsRepository;
pub use sources::{NewSource, SourceRecord, SourceRepository};
pub use storage::StorageAccountingRepository;
