use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::utils::lock::GrainedLock;

use super::query::{Access, RetreivalContainer, Retrieved};

pub trait ResourceContainer {
    fn add_resource<T: 'static>(&self, resource: T);
    fn add_resource_any(&self, type_id: TypeId, resource: Box<dyn Any>);
    fn remove_resource<T: 'static>(&self) -> Option<T>;
    fn remove_resource_any(&self, type_id: TypeId) -> Option<Box<dyn Any>>;
    fn contains_resource<T: 'static>(&self) -> bool;
    fn contains_resource_any(&self, type_id: TypeId) -> bool;
}

#[derive(Debug)]
pub struct Container {
    resources: GrainedLock<Box<dyn Any>>,
}

impl Default for Container {
    fn default() -> Self {
        Self {
            resources: GrainedLock::new(Box::new(
                HashMap::<TypeId, GrainedLock<Box<dyn Any>>>::new(),
            )),
        }
    }
}

impl ResourceContainer for Container {
    fn add_resource<T: 'static>(&self, resource: T) {
        self.add_resource_any(TypeId::of::<T>(), Box::new(resource));
    }

    fn add_resource_any(&self, type_id: TypeId, resource: Box<dyn Any>) {
        self.resources
            .borrow_mut()
            .downcast_mut::<HashMap<TypeId, GrainedLock<Box<dyn Any>>>>()
            .unwrap()
            .insert(type_id, GrainedLock::new(resource));
    }

    fn remove_resource<T: 'static>(&self) -> Option<T> {
        self.remove_resource_any(TypeId::of::<T>())
            .map(|b| *b.downcast::<T>().unwrap())
    }

    fn remove_resource_any(&self, type_id: TypeId) -> Option<Box<dyn Any>> {
        self.resources
            .borrow_mut()
            .downcast_mut::<HashMap<TypeId, GrainedLock<Box<dyn Any>>>>()
            .unwrap()
            .remove(&type_id)
            .map(|b| b.take())
    }

    fn contains_resource<T: 'static>(&self) -> bool {
        self.contains_resource_any(TypeId::of::<T>())
    }

    fn contains_resource_any(&self, type_id: TypeId) -> bool {
        self.resources
            .borrow()
            .downcast_ref::<HashMap<TypeId, GrainedLock<Box<dyn Any>>>>()
            .unwrap()
            .contains_key(&type_id)
    }
}

impl RetreivalContainer for Container {
    fn get<'a>(&'a self, type_id: TypeId, access: Access) -> super::query::Retrieved<'a> {
        if type_id == TypeId::of::<HashMap<TypeId, GrainedLock<Box<dyn Any>>>>() {
            match access {
                Access::Immutable => Retrieved::Immutable(self.resources.borrow()),
                Access::Mutable => Retrieved::Mutable(self.resources.borrow_mut()),
            }
        } else {
            if self
                .resources
                .borrow()
                .downcast_ref::<HashMap<TypeId, GrainedLock<Box<dyn Any>>>>()
                .unwrap()
                .contains_key(&type_id)
            {
                let resource = self.resources.borrow().map_cell(|resources| {
                    resources
                        .downcast_ref::<HashMap<TypeId, GrainedLock<Box<dyn Any>>>>()
                        .unwrap()
                        .get(&type_id)
                        .unwrap()
                });
                match access {
                    Access::Immutable => Retrieved::Immutable(resource.borrow()),
                    Access::Mutable => Retrieved::Mutable(resource.borrow_mut()),
                }
            } else {
                Retrieved::NotFound
            }
        }
    }
}

#[cfg(test)]
mod test_container {
    use std::{any::{Any, TypeId}, collections::HashMap};

    use crate::{store::{
        query::{Access, RetreivalContainer},
        ResourceContainer,
    }, utils::lock::GrainedLock};

    use super::Container;

    #[test]
    fn test_default() {
        let _ = Container::default();
    }

    #[test]
    fn test_add_resource() {
        let container = Container::default();
        container.add_resource(1);
    }

    #[test]
    fn test_remove_resource() {
        let container = Container::default();
        container.add_resource(1i32);
        assert!(container.remove_resource::<i32>().is_some());
        assert!(container.remove_resource::<i64>().is_none());
    }

    #[test]
    fn test_contains_resource() {
        let container = Container::default();
        container.add_resource(1i32);
        assert!(container.contains_resource::<i32>());
        assert!(!container.contains_resource::<i64>());
    }

    #[test]
    fn test_get() {
        let container = Container::default();
        container.add_resource(1i32);
        {
            let resource = container.get(TypeId::of::<i32>(), Access::Immutable);
            assert!(resource.is_found());
            assert!(resource.is_immutable());
        }

        {
            let resource = container.get(TypeId::of::<i32>(), Access::Mutable);
            assert!(resource.is_found());
            assert!(resource.is_mutable());
        }

        {
            let resource = container.get(TypeId::of::<i64>(), Access::Immutable);
            assert!(!resource.is_found());
            assert!(!resource.is_immutable());
            assert!(!resource.is_mutable());
        }
    }

    #[test]
    fn test_get_hashmap() {
        let container = Container::default();
        container.add_resource(1i32);
        let resource = container.get(
            TypeId::of::<HashMap<TypeId, GrainedLock<Box<dyn Any>>>>(),
            Access::Immutable,
        );
        assert!(resource.is_found());
    }
}
