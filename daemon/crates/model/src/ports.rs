use crate::domain::{Model, Provider};

pub trait ModelCatalog: Send + Sync {
    fn list(&self) -> Vec<Model>;
    fn get(&self, id: &str) -> Option<Model>;
    fn provider(&self, id: &str) -> Option<Provider>;
}
