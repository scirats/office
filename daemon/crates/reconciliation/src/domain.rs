#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedWorkspace {
    pub slug: String,
    pub user: String,
    pub home: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceObservation {
    pub user_exists: bool,
    pub user_uid: Option<u32>,
    pub home_exists: bool,
    pub home_uid: Option<u32>,
    pub home_mode: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inconsistency {
    MissingUser,
    MissingHome,
    WrongOwner,
    WrongMode,
    InspectionUnavailable,
}

impl Inconsistency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Inconsistency::MissingUser => "missing_user",
            Inconsistency::MissingHome => "missing_home",
            Inconsistency::WrongOwner => "wrong_owner",
            Inconsistency::WrongMode => "wrong_mode",
            Inconsistency::InspectionUnavailable => "inspection_unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceStatus {
    Active,
    Inconsistent,
}

impl WorkspaceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkspaceStatus::Active => "active",
            WorkspaceStatus::Inconsistent => "inconsistent",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceReconciliation {
    pub slug: String,
    pub status: WorkspaceStatus,
    pub inconsistencies: Vec<Inconsistency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReconciliationReport {
    pub workspaces: Vec<WorkspaceReconciliation>,
}
