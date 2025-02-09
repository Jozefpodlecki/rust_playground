use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::sync::Arc;

pub struct ServiceRegistry {
    services: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn register<T: Any + 'static + Send + Sync>(&mut self, service: Arc<T>) {
        self.services.insert(TypeId::of::<T>(), service);
    }

    pub fn get_required<T: Send + Sync + 'static>(&self) -> Arc<dyn Any + Send + Sync> {
        self.services
            .get(&TypeId::of::<T>())
            .unwrap()
            .clone()
    }
}
