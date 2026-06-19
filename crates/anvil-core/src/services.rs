use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone, Default)]
pub struct Services {
    map: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Services {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, value: T) {
        self.map.insert(TypeId::of::<T>(), Arc::new(value));
    }

    pub fn get<T: Any + Send + Sync + Clone>(&self) -> Option<T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    }
}

pub trait FromServices: Sized {
    fn from_services(services: &Services) -> Result<Self, BoxError>;
}

impl<T: Any + Send + Sync + Clone> FromServices for T {
    fn from_services(services: &Services) -> Result<Self, BoxError> {
        services.get::<T>().ok_or_else(|| {
            format!(
                "service `{}` is not registered in AppState",
                std::any::type_name::<T>()
            )
            .into()
        })
    }
}
