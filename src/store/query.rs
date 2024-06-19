use std::any::{Any, TypeId};

use crate::utils::lock::{
    grained_ref::{Immutable, LockState, Mutable},
    Ref,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Access {
    Immutable,
    Mutable,
}

impl From<Immutable> for Access {
    fn from(_: Immutable) -> Self {
        Self::Immutable
    }
}

impl From<Mutable> for Access {
    fn from(_: Mutable) -> Self {
        Self::Mutable
    }
}

pub enum Retrieved<'a> {
    Immutable(Ref<'a, Box<dyn Any>, Immutable>),
    Mutable(Ref<'a, Box<dyn Any>, Mutable>),
    NotFound,
}

impl Retrieved<'_> {
    pub fn is_found(&self) -> bool {
        matches!(self, Retrieved::Immutable(_) | Retrieved::Mutable(_))
    }

    pub fn is_immutable(&self) -> bool {
        matches!(self, Retrieved::Immutable(_))
    }

    pub fn is_mutable(&self) -> bool {
        matches!(self, Retrieved::Mutable(_))
    }
}

pub trait RetreivalContainer {
    fn get<'a>(&'a self, type_id: TypeId, access: Access) -> Retrieved<'a>;
}

pub trait Retrievable {
    type Access: LockState;
    type Item<'a>;

    fn type_id() -> TypeId;
    fn from_retrieved<'a>(retrieved: Retrieved<'a>) -> Self::Item<'a>;
}

pub trait Retriever {
    type Item<'a>;

    fn retrieve<'a>(container: &'a impl RetreivalContainer) -> Self::Item<'a>;
}

macro_rules! count_tts {
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $_k:tt $_l:tt $_m:tt $_n:tt $_o:tt
     $_p:tt $_q:tt $_r:tt $_s:tt $_t:tt
     $($tail:tt)*)
        => {20usize + count_tts!($($tail)*)};
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $($tail:tt)*)
        => {10usize + count_tts!($($tail)*)};
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $($tail:tt)*)
        => {5usize + count_tts!($($tail)*)};
    ($_a:tt
     $($tail:tt)*)
        => {1usize + count_tts!($($tail)*)};
    () => {0usize};
}

macro_rules! impl_retrievable {
    ($($param:ident),*) => {
        impl<$($param : Retrievable,)*> Retriever for ($($param,)*)
        where
            $(
                Access: From<<$param as Retrievable>::Access>,
                <$param as Retrievable>::Access : Default,
            )*
        {
            type Item<'a> = ($($param::Item<'a>,)*);

            fn retrieve<'a>(container: &'a impl RetreivalContainer) -> Self::Item<'a> {
                // get param length
                const LENGTH:usize = count_tts!($($param)*);

                // get type id
                // note that index of type_ids doesnt have correct value yet
                let mut type_ids:[(TypeId, Access, usize); LENGTH] = [
                    $(
                        ($param::type_id(), Access::from(<$param as Retrievable>::Access::default()), 0),
                    )*
                ];

                // fill index with correct value
                for i in 0..LENGTH {
                    type_ids[i].2 = i;
                }

                // sort by type_id
                type_ids.sort_by(|a, b| a.0.cmp(&b.0));

                // create a stub array to retrieve
                let mut cells:[(TypeId, Option<Retrieved>, usize); LENGTH] = [
                    $(
                        ($param::type_id(), None, 0),
                    )*
                ];

                // fill stub array with correct value
                for (i, (type_id, access, index)) in type_ids.into_iter().enumerate() {
                    cells[i] = (type_id, Some(container.get(type_id, access)), index);
                }

                // sort by index
                cells.sort_by(|a, b| a.2.cmp(&b.2));

                // turns into iter
                let mut iter = cells.into_iter();

                // return
                (
                    $(
                        $param::from_retrieved(iter.next().unwrap().1.unwrap()),
                    )*
                )
            }

        }
    };
}

impl_retrievable!(T1);
impl_retrievable!(T1, T2);
impl_retrievable!(T1, T2, T3);
impl_retrievable!(T1, T2, T3, T4);
impl_retrievable!(T1, T2, T3, T4, T5);
impl_retrievable!(T1, T2, T3, T4, T5, T6);
impl_retrievable!(T1, T2, T3, T4, T5, T6, T7);
impl_retrievable!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_retrievable!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_retrievable!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_retrievable!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_retrievable!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_retrievable!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_retrievable!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_retrievable!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_retrievable!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
