use crate::domain::{Workspace, WorkspaceRecord, is_valid_slug, slugify};
use crate::ports::{ProvisionError, StoreError, UserProvisioner, WorkspaceRepository};

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("invalid workspace name")]
    InvalidName,
    #[error("workspace not found: {0}")]
    UnknownWorkspace(String),
    #[error(transparent)]
    Provision(#[from] ProvisionError),
    #[error(transparent)]
    Store(#[from] StoreError),
}

pub struct WorkspaceService {
    provisioner: Box<dyn UserProvisioner>,
    repository: Box<dyn WorkspaceRepository>,
}

impl WorkspaceService {
    pub fn new(
        provisioner: Box<dyn UserProvisioner>,
        repository: Box<dyn WorkspaceRepository>,
    ) -> Self {
        Self {
            provisioner,
            repository,
        }
    }

    pub fn create(&self, name: &str) -> Result<Workspace, WorkspaceError> {
        let slug = slugify(name);
        if !is_valid_slug(&slug) {
            return Err(WorkspaceError::InvalidName);
        }

        let provisioned = self.provisioner.provision(&slug)?;
        let workspace = Workspace {
            name: name.trim().to_owned(),
            slug,
            user: provisioned.user,
            home: provisioned.home,
        };

        self.repository.save(&workspace)?;
        Ok(workspace)
    }

    pub fn list(&self) -> Result<Vec<WorkspaceRecord>, WorkspaceError> {
        Ok(self.repository.list()?)
    }

    pub fn delete(&self, slug: &str) -> Result<WorkspaceRecord, WorkspaceError> {
        let record = self
            .repository
            .find(slug)?
            .ok_or_else(|| WorkspaceError::UnknownWorkspace(slug.to_owned()))?;

        self.provisioner.deprovision(&record.user, &record.home)?;
        self.repository.remove(slug)?;
        Ok(record)
    }

    pub fn set_status(&self, slug: &str, status: &str) -> Result<(), WorkspaceError> {
        self.repository.set_status(slug, status)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::Provisioned;
    use std::sync::Mutex;

    struct FakeProvisioner {
        calls: Mutex<Vec<String>>,
    }

    impl FakeProvisioner {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
            }
        }
    }

    impl UserProvisioner for FakeProvisioner {
        fn provision(&self, slug: &str) -> Result<Provisioned, ProvisionError> {
            self.calls.lock().unwrap().push(slug.to_owned());
            Ok(Provisioned {
                user: slug.to_owned(),
                home: format!("/home/{slug}"),
            })
        }

        fn deprovision(&self, user: &str, _home: &str) -> Result<(), ProvisionError> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("deprovision:{user}"));
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeRepository {
        records: Mutex<Vec<WorkspaceRecord>>,
    }

    impl WorkspaceRepository for FakeRepository {
        fn save(&self, workspace: &Workspace) -> Result<(), StoreError> {
            self.records.lock().unwrap().push(WorkspaceRecord {
                name: workspace.name.clone(),
                slug: workspace.slug.clone(),
                user: workspace.user.clone(),
                home: workspace.home.clone(),
                created_at: "2026-01-01T00:00:00Z".to_owned(),
                status: "active".to_owned(),
            });
            Ok(())
        }

        fn find(&self, slug: &str) -> Result<Option<WorkspaceRecord>, StoreError> {
            Ok(self
                .records
                .lock()
                .unwrap()
                .iter()
                .find(|record| record.slug == slug)
                .cloned())
        }

        fn list(&self) -> Result<Vec<WorkspaceRecord>, StoreError> {
            Ok(self.records.lock().unwrap().clone())
        }

        fn set_status(&self, slug: &str, status: &str) -> Result<(), StoreError> {
            for record in self.records.lock().unwrap().iter_mut() {
                if record.slug == slug {
                    record.status = status.to_owned();
                }
            }
            Ok(())
        }

        fn remove(&self, slug: &str) -> Result<bool, StoreError> {
            let mut records = self.records.lock().unwrap();
            let before = records.len();
            records.retain(|record| record.slug != slug);
            Ok(records.len() != before)
        }
    }

    fn service() -> WorkspaceService {
        WorkspaceService::new(
            Box::new(FakeProvisioner::new()),
            Box::new(FakeRepository::default()),
        )
    }

    #[test]
    fn creates_and_persists_workspace() {
        let service = service();
        let workspace = service.create("Acme Corp").expect("creates");

        assert_eq!(workspace.name, "Acme Corp");
        assert_eq!(workspace.slug, "acme-corp");
        assert_eq!(workspace.user, "acme-corp");
        assert_eq!(workspace.home, "/home/acme-corp");

        let records = service.list().expect("lists");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].slug, "acme-corp");
    }

    #[test]
    fn rejects_name_that_yields_no_slug() {
        let service = service();
        assert!(matches!(
            service.create("!!!"),
            Err(WorkspaceError::InvalidName)
        ));
    }

    #[test]
    fn surfaces_provisioning_errors() {
        struct Unsupported;
        impl UserProvisioner for Unsupported {
            fn provision(&self, _slug: &str) -> Result<Provisioned, ProvisionError> {
                Err(ProvisionError::Unsupported)
            }

            fn deprovision(&self, _user: &str, _home: &str) -> Result<(), ProvisionError> {
                Err(ProvisionError::Unsupported)
            }
        }

        let service =
            WorkspaceService::new(Box::new(Unsupported), Box::new(FakeRepository::default()));
        assert!(matches!(
            service.create("acme"),
            Err(WorkspaceError::Provision(ProvisionError::Unsupported))
        ));
    }

    #[test]
    fn deletes_existing_workspace_and_deprovisions() {
        let service = service();
        service.create("Acme Corp").expect("creates");

        let removed = service.delete("acme-corp").expect("deletes");
        assert_eq!(removed.slug, "acme-corp");
        assert_eq!(removed.user, "acme-corp");
        assert!(service.list().expect("lists").is_empty());
    }

    #[test]
    fn rejects_deleting_unknown_workspace() {
        let service = service();
        assert!(matches!(
            service.delete("ghost"),
            Err(WorkspaceError::UnknownWorkspace(slug)) if slug == "ghost"
        ));
    }

    #[test]
    fn keeps_record_when_deprovision_fails() {
        struct Failing;
        impl UserProvisioner for Failing {
            fn provision(&self, _slug: &str) -> Result<Provisioned, ProvisionError> {
                Err(ProvisionError::Unsupported)
            }

            fn deprovision(&self, user: &str, _home: &str) -> Result<(), ProvisionError> {
                Err(ProvisionError::Command {
                    action: "userdel".to_owned(),
                    target: user.to_owned(),
                    message: "user is currently used by process".to_owned(),
                })
            }
        }

        let repository = FakeRepository::default();
        repository
            .save(&Workspace {
                name: "Acme".to_owned(),
                slug: "acme".to_owned(),
                user: "acme".to_owned(),
                home: "/home/acme".to_owned(),
            })
            .expect("saves");

        let service = WorkspaceService::new(Box::new(Failing), Box::new(repository));
        assert!(matches!(
            service.delete("acme"),
            Err(WorkspaceError::Provision(_))
        ));
        assert_eq!(service.list().expect("lists").len(), 1);
    }
}
