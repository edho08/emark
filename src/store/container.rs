use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::utils::lock::GrainedLock;

pub trait Container {
    fn add_resource<T: 'static>(&mut self, resource: T);
    fn add_resource_any(&mut self, type_id: TypeId, resource: Box<dyn Any>);
    fn remove_resource<T: 'static>(&mut self) -> Option<T>;
    fn remove_resource_any(&mut self, type_id: TypeId) -> Option<Box<dyn Any>>;
    fn contains_resource<T: 'static>(&self) -> bool;
    fn contains_resource_any(&self, type_id: TypeId) -> bool;
}

#[derive(Debug, Default)]
pub struct ResourceContainer {
    resources: HashMap<TypeId, GrainedLock<Box<dyn Any>>>,
}

impl Container for ResourceContainer {
    fn add_resource<T: 'static>(&mut self, resource: T) {
        self.add_resource_any(TypeId::of::<T>(), Box::new(resource));
    }

    fn add_resource_any(&mut self, type_id: TypeId, resource: Box<dyn Any>) {
        self.resources.insert(type_id, GrainedLock::new(resource));
    }

    fn remove_resource<T: 'static>(&mut self) -> Option<T> {
        self.remove_resource_any(TypeId::of::<T>())
            .map(|resource| *resource.downcast::<T>().unwrap())
    }

    fn remove_resource_any(&mut self, type_id: TypeId) -> Option<Box<dyn Any>> {
        self.resources
            .remove(&type_id)
            .map(|resource| resource.take())
    }

    fn contains_resource<T: 'static>(&self) -> bool {
        self.contains_resource_any(TypeId::of::<T>())
    }

    fn contains_resource_any(&self, type_id: TypeId) -> bool {
        self.resources.contains_key(&type_id)
    }
}

#[cfg(test)]
mod test_container {
    use super::*;

    #[test]
    fn test_container_add_contains() {
        let mut container = ResourceContainer::default();
        container.add_resource(1);
        assert!(container.contains_resource::<i32>());
    }

    #[test]
    fn test_remove_resource() {
        let mut container = ResourceContainer::default();
        container.add_resource(1);
        let removed = container.remove_resource::<i32>();
        assert!(!container.contains_resource::<i32>());
        assert_eq!(removed, Some(1));
    }

    #[test]
    fn test_add_resource_any() {
        let mut container = ResourceContainer::default();
        container.add_resource_any(TypeId::of::<i32>(), Box::new(1));
        assert!(container.contains_resource::<i32>());
    }

    #[test]
    fn test_remove_resource_any() {
        let mut container = ResourceContainer::default();
        container.add_resource_any(TypeId::of::<i32>(), Box::new(1));
        let removed = container.remove_resource_any(TypeId::of::<i32>());
        assert!(!container.contains_resource::<i32>());
        assert_eq!(
            removed.map(|removed| removed.downcast::<i32>().unwrap()),
            Some(Box::new(1))
        );
    }
}
