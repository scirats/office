pub mod domain;
pub mod ports;
pub mod service;

pub use domain::{Project, ProjectId, ProjectRecord, is_valid_slug, slugify};
pub use ports::{ProjectRepository, StoreError};
pub use service::{ProjectError, ProjectService};
