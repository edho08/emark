use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::utils::lock::{
    grained_ref::{Immutable, Mutable},
    Ref,
};

use super::query::{Access, Retrievable, Retriever};

pub struct Res<'a, T>(Ref<'a, Box<dyn Any>, Immutable>, PhantomData<T>);
pub struct ResMut<'a, T>(Ref<'a, Box<dyn Any>, Mutable>, PhantomData<T>);
pub struct ResClone<T>(T)
where
    T: 'static + Clone;

impl<T> Deref for Res<'_, T>
where
    T: 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.downcast_ref().unwrap()
    }
}

impl<T> Deref for ResMut<'_, T>
where
    T: 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.downcast_ref().unwrap()
    }
}

impl<T> Deref for ResClone<T>
where
    T: Clone,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ResClone<T>
where
    T: Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> DerefMut for ResMut<'_, T>
where
    T: 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.downcast_mut().unwrap()
    }
}

impl<T> AsRef<T> for Res<'_, T>
where
    T: 'static,
{
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> AsRef<T> for ResMut<'_, T>
where
    T: 'static,
{
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> AsMut<T> for ResMut<'_, T>
where
    T: 'static,
{
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<T> Retrievable for Res<'_, T>
where
    T: 'static,
{
    type Access = Immutable;
    type Item<'a> = Res<'a, T>;

    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn from_retrieved<'a>(retrieved: super::query::Retrieved<'a>) -> Self::Item<'a> {
        match retrieved {
            super::query::Retrieved::Immutable(immutable) => Res(immutable, PhantomData),
            _ => unreachable!(),
        }
    }
}

impl<T> Retrievable for ResMut<'_, T>
where
    T: 'static,
{
    type Access = Mutable;
    type Item<'a> = ResMut<'a, T>;

    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn from_retrieved<'a>(retrieved: super::query::Retrieved<'a>) -> Self::Item<'a> {
        match retrieved {
            super::query::Retrieved::Mutable(mutable) => ResMut(mutable, PhantomData),
            _ => unreachable!(),
        }
    }
}

impl<T> Retrievable for ResClone<T>
where
    T: Clone,
{
    type Access = Immutable;
    type Item<'a> = ResClone<T>;

    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn from_retrieved<'a>(retrieved: super::query::Retrieved<'a>) -> Self::Item<'a> {
        match retrieved {
            super::query::Retrieved::Immutable(immutable) => {
                ResClone(immutable.downcast_ref::<T>().unwrap().clone())
            }
            _ => unreachable!(),
        }
    }
}

impl<'b, T> Retrievable for Option<Res<'b, T>>
where
    T: 'static,
    Res<'b, T>: Retrievable,
{
    type Access = Immutable;
    type Item<'a> = Option<Res<'a, T>>;

    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn from_retrieved<'a>(retrieved: super::query::Retrieved<'a>) -> Self::Item<'a> {
        match retrieved {
            super::query::Retrieved::Immutable(immutable) => Some(Res(immutable, PhantomData)),
            _ => None,
        }
    }
}

impl<'b, T> Retrievable for Option<ResMut<'b, T>>
where
    T: 'static,
    ResMut<'b, T>: Retrievable,
{
    type Access = Mutable;
    type Item<'a> = Option<ResMut<'a, T>>;

    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn from_retrieved<'a>(retrieved: super::query::Retrieved<'a>) -> Self::Item<'a> {
        match retrieved {
            super::query::Retrieved::Mutable(mutable) => Some(ResMut(mutable, PhantomData)),
            _ => None,
        }
    }
}

impl<'b, T> Retriever for Res<'b, T>
where
    T: 'static,
    Res<'b, T>: Retrievable,
{
    type Item<'a> = Res<'a, T>;

    fn retrieve<'a>(container: &'a impl super::query::RetreivalContainer) -> Self::Item<'a> {
        Res(
            match container.get(TypeId::of::<T>(), Access::from(Immutable)) {
                super::query::Retrieved::Immutable(value) => value,
                super::query::Retrieved::Mutable(_) => unreachable!(),
                super::query::Retrieved::NotFound => panic!("Resource not found"),
            },
            PhantomData,
        )
    }
}

impl<'b, T> Retriever for ResMut<'b, T>
where
    T: 'static,
    ResMut<'b, T>: Retrievable,
{
    type Item<'a> = ResMut<'a, T>;

    fn retrieve<'a>(container: &'a impl super::query::RetreivalContainer) -> Self::Item<'a> {
        ResMut(
            match container.get(TypeId::of::<T>(), Access::from(Mutable)) {
                super::query::Retrieved::Immutable(_) => unreachable!(),
                super::query::Retrieved::Mutable(value) => value,
                super::query::Retrieved::NotFound => panic!("Resource not found"),
            },
            PhantomData,
        )
    }
}

impl<T> Retriever for ResClone<T>
where
    T: Clone,
{
    type Item<'a> = ResClone<T>;

    fn retrieve<'a>(container: &'a impl super::query::RetreivalContainer) -> Self::Item<'a> {
        ResClone(
            match container.get(TypeId::of::<T>(), Access::from(Immutable)) {
                super::query::Retrieved::Immutable(value) => {
                    value.downcast_ref::<T>().unwrap().clone()
                }
                super::query::Retrieved::Mutable(_) => unreachable!(),
                super::query::Retrieved::NotFound => panic!("Resource not found"),
            },
        )
    }
}

impl<'b, T> Retriever for Option<Res<'b, T>>
where
    T: 'static,
    Res<'b, T>: Retrievable,
{
    type Item<'a> = Option<Res<'a, T>>;

    fn retrieve<'a>(container: &'a impl super::query::RetreivalContainer) -> Self::Item<'a> {
        match container.get(TypeId::of::<T>(), Access::from(Immutable)) {
            super::query::Retrieved::Immutable(value) => Some(Res(value, PhantomData)),
            _ => None,
        }
    }
}

impl<'b, T> Retriever for Option<ResMut<'b, T>>
where
    T: 'static,
    ResMut<'b, T>: Retrievable,
{
    type Item<'a> = Option<ResMut<'a, T>>;

    fn retrieve<'a>(container: &'a impl super::query::RetreivalContainer) -> Self::Item<'a> {
        match container.get(TypeId::of::<T>(), Access::from(Mutable)) {
            super::query::Retrieved::Mutable(value) => Some(ResMut(value, PhantomData)),
            _ => None,
        }
    }
}

impl<T> Retriever for Option<ResClone<T>>
where
    T: 'static + Clone,
    ResClone<T>: Retrievable,
{
    type Item<'a> = Option<ResClone<T>>;

    fn retrieve<'a>(container: &'a impl super::query::RetreivalContainer) -> Self::Item<'a> {
        match container.get(TypeId::of::<T>(), Access::from(Immutable)) {
            super::query::Retrieved::Immutable(value) => {
                Some(ResClone(value.downcast_ref::<T>().unwrap().clone()))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests_res {
    use std::collections::HashMap;

    use crate::{store::{Container, ResourceContainer}, utils::lock::GrainedLock};

    use super::*;

    #[test]
    fn test_res() {
        // create container
        let container = Container::default();
        // add resource
        container.add_resource(1i32);
        // get resource
        let res = Res::<i32>::retrieve(&container);
        // assert value
        assert_eq!(*res, 1i32);
    }

    #[test]
    fn test_res_mut() {
        // create container
        let container = Container::default();
        // add resource
        container.add_resource(1i32);
        // get resource
        let res = ResMut::<i32>::retrieve(&container);
        // assert value
        assert_eq!(*res, 1i32);
    }

    #[test]
    #[should_panic]
    fn test_res_not_found() {
        // create container
        let container = Container::default();
        // get resource
        let _ = Res::<i32>::retrieve(&container);
    }

    #[test]
    #[should_panic]
    fn test_res_mut_not_found() {
        // create container
        let container = Container::default();
        // get resource
        let _ = ResMut::<i32>::retrieve(&container);
    }

    #[test]
    fn test_res_as_ref() {
        // create container
        let container = Container::default();
        // add resource
        container.add_resource(1i32);
        // get resource
        let res = Res::<i32>::retrieve(&container);
        // assert value
        assert_eq!(*res.as_ref(), 1i32);
    }

    #[test]
    fn test_res_mut_as_ref() {
        // create container
        let container = Container::default();
        // add resource
        container.add_resource(1i32);
        // get resource
        let res = ResMut::<i32>::retrieve(&container);
        // assert value
        assert_eq!(*res.as_ref(), 1i32);
    }

    #[test]
    fn test_res_multiple() {
        // create container
        let container = Container::default();
        // add resource
        container.add_resource(1i32);
        container.add_resource(1u32);
        // get resources
        let (res1, res2) = <(Res<i32>, Res<u32>)>::retrieve(&container);
        // assert value
        assert_eq!(*res1, 1i32);
        assert_eq!(*res2, 1u32);
    }

    #[test]
    fn test_res_mut_multiple() {
        // create container
        let container = Container::default();
        // add resource
        container.add_resource(1i32);
        container.add_resource(1u32);
        // get resources
        let (res1, res2) = <(ResMut<i32>, ResMut<u32>)>::retrieve(&container);
        // assert value
        assert_eq!(*res1, 1i32);
        assert_eq!(*res2, 1u32);
    }

    #[test]
    fn test_res_clone() {
        // create container
        let container = Container::default();
        // add resource
        container.add_resource(1i32);
        // get resource
        let res = ResClone::<i32>::retrieve(&container);
        // assert value
        assert_eq!(*res, 1i32);
    }

    #[test]
    fn test_res_option() {
        // create container
        let container = Container::default();
        // add resource
        container.add_resource(1i32);
        // get resource
        let res = Option::<Res<i32>>::retrieve(&container);
        // assert value
        assert_eq!(*res.unwrap(), 1i32);
    }

    #[test]
    fn test_res_option_none() {
        // create container
        let container = Container::default();
        // get resource
        let res = Option::<Res<i32>>::retrieve(&container);
        // assert value
        assert!(res.is_none());
    }

    #[test]
    fn test_res_option_clone() {
        // create container
        let container = Container::default();
        // add resource
        container.add_resource(1i32);
        // get resource
        let res = Option::<ResClone<i32>>::retrieve(&container);
        // assert value
        assert_eq!(*res.unwrap(), 1i32);
    }

    #[test]
    fn test_res_option_clone_none() {
        // create container
        let container = Container::default();
        // get resource
        let res = Option::<ResClone<i32>>::retrieve(&container);
        // assert value
        assert!(res.is_none());
    }

    #[test]
    fn test_res_option_mut() {
        // create container
        let container = Container::default();
        // add resource
        container.add_resource(1i32);
        // get resource
        let res = Option::<ResMut<i32>>::retrieve(&container);
        // assert value
        assert_eq!(*res.unwrap(), 1i32);
    }

    #[test]
    fn test_res_option_mut_none() {
        // create container
        let container = Container::default();
        // get resource
        let res = Option::<ResMut<i32>>::retrieve(&container);
        // assert value
        assert!(res.is_none());
    }

    #[test]
    fn test_resmut_change_value() {
        // create container
        let container = Container::default();
        // add resource
        container.add_resource(1i32);
        {
            // get resource
            let mut res = ResMut::<i32>::retrieve(&container);
            // change value
            *res = 2i32;
            // assert value
            assert_eq!(*res, 2i32);
        }

        // reborrow
        let res = Res::<i32>::retrieve(&container);
        // assert value
        assert_eq!(*res, 2i32);
    }

    #[test]
    fn test_res_hashmap(){
        // create container
        let container = Container::default();
        // get resource
        let _ = Res::<HashMap::<TypeId, GrainedLock<Box<dyn Any>>>>::retrieve(&container);
    }
}
