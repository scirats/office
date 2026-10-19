use crate::domain::{Project, ProjectId, ProjectRecord, is_valid_slug, slugify};
use crate::ports::{ProjectRepository, StoreError};

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("project name does not yield a valid id")]
    InvalidName,
    #[error("workspace is not a valid slug")]
    InvalidWorkspace,
    #[error("project '{0}/{1}' already exists")]
    AlreadyExists(String, String),
    #[error("project '{workspace}/{id}' not found")]
    UnknownProject { workspace: String, id: String },
    #[error(transparent)]
    Store(StoreError),
}

impl From<StoreError> for ProjectError {
    fn from(error: StoreError) -> Self {
        ProjectError::Store(error)
    }
}

pub struct ProjectService {
    repository: Box<dyn ProjectRepository>,
}

impl ProjectService {
    pub fn new(repository: Box<dyn ProjectRepository>) -> Self {
        Self { repository }
    }

    pub fn create(&self, workspace: &str, name: &str) -> Result<Project, ProjectError> {
        if !is_valid_slug(workspace) {
            return Err(ProjectError::InvalidWorkspace);
        }
        let id = slugify(name);
        if !is_valid_slug(&id) {
            return Err(ProjectError::InvalidName);
        }

        let project = Project {
            id: ProjectId(id),
            workspace: workspace.to_owned(),
            name: name.trim().to_owned(),
        };

        match self.repository.save(&project) {
            Ok(()) => Ok(project),
            Err(StoreError::Conflict) => {
                Err(ProjectError::AlreadyExists(project.workspace, project.id.0))
            }
            Err(other) => Err(ProjectError::Store(other)),
        }
    }

    pub fn list(&self, workspace: Option<&str>) -> Result<Vec<ProjectRecord>, ProjectError> {
        Ok(self.repository.list(workspace)?)
    }

    pub fn delete(&self, workspace: &str, id: &str) -> Result<ProjectRecord, ProjectError> {
        let record =
            self.repository
                .find(workspace, id)?
                .ok_or_else(|| ProjectError::UnknownProject {
                    workspace: workspace.to_owned(),
                    id: id.to_owned(),
                })?;
        self.repository.remove(workspace, id)?;
        Ok(record)
    }

    pub fn exists(&self, workspace: &str, id: &str) -> Result<bool, ProjectError> {
        Ok(self.repository.find(workspace, id)?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeRepository {
        records: Mutex<Vec<ProjectRecord>>,
    }

    impl ProjectRepository for FakeRepository {
        fn save(&self, project: &Project) -> Result<(), StoreError> {
            let mut records = self.records.lock().unwrap();
            let duplicate = records
                .iter()
                .any(|item| item.workspace == project.workspace && item.id == project.id);
            if duplicate {
                return Err(StoreError::Conflict);
            }
            records.push(ProjectRecord {
                id: project.id.clone(),
                workspace: project.workspace.clone(),
                name: project.name.clone(),
                created_at: "2026-01-01T00:00:00Z".to_owned(),
            });
            Ok(())
        }

        fn find(&self, workspace: &str, id: &str) -> Result<Option<ProjectRecord>, StoreError> {
            Ok(self
                .records
                .lock()
                .unwrap()
                .iter()
                .find(|item| item.workspace == workspace && item.id.as_str() == id)
                .cloned())
        }

        fn list(&self, workspace: Option<&str>) -> Result<Vec<ProjectRecord>, StoreError> {
            Ok(self
                .records
                .lock()
                .unwrap()
                .iter()
                .filter(|item| workspace.is_none_or(|value| item.workspace == value))
                .cloned()
                .collect())
        }

        fn remove(&self, workspace: &str, id: &str) -> Result<bool, StoreError> {
            let mut records = self.records.lock().unwrap();
            let before = records.len();
            records.retain(|item| !(item.workspace == workspace && item.id.as_str() == id));
            Ok(records.len() != before)
        }
    }

    fn service() -> ProjectService {
        ProjectService::new(Box::new(FakeRepository::default()))
    }

    #[test]
    fn creates_project_with_slug_and_workspace() {
        let service = service();
        let project = service
            .create("acme", "Ecommerce Backend")
            .expect("creates");

        assert_eq!(project.id.as_str(), "ecommerce-backend");
        assert_eq!(project.workspace, "acme");
        assert_eq!(project.name, "Ecommerce Backend");
        assert_eq!(service.list(Some("acme")).unwrap().len(), 1);
        assert!(service.exists("acme", "ecommerce-backend").unwrap());
    }

    #[test]
    fn rejects_invalid_name_and_workspace() {
        let service = service();
        assert!(matches!(
            service.create("acme", "!!!"),
            Err(ProjectError::InvalidName)
        ));
        assert!(matches!(
            service.create("Acme", "ecommerce"),
            Err(ProjectError::InvalidWorkspace)
        ));
    }

    #[test]
    fn rejects_duplicate_project_in_same_workspace() {
        let service = service();
        service.create("acme", "ecommerce").expect("creates");
        assert!(matches!(
            service.create("acme", "ecommerce"),
            Err(ProjectError::AlreadyExists(workspace, id))
                if workspace == "acme" && id == "ecommerce"
        ));
    }

    #[test]
    fn allows_same_project_name_in_different_workspaces() {
        let service = service();
        service.create("acme", "ecommerce").expect("creates");
        service.create("foo", "ecommerce").expect("creates");

        assert_eq!(service.list(None).unwrap().len(), 2);
        assert_eq!(service.list(Some("foo")).unwrap().len(), 1);
    }

    #[test]
    fn deletes_existing_project_and_reports_unknown() {
        let service = service();
        service.create("acme", "ecommerce").expect("creates");

        let removed = service.delete("acme", "ecommerce").expect("deletes");
        assert_eq!(removed.id.as_str(), "ecommerce");
        assert!(service.list(None).unwrap().is_empty());
        assert!(matches!(
            service.delete("acme", "ghost"),
            Err(ProjectError::UnknownProject { workspace, id })
                if workspace == "acme" && id == "ghost"
        ));
    }
}
