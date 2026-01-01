use std::{
    collections::{HashSet, hash_set::IntoIter},
    hash::BuildHasher,
    iter::Once,
};
use transmission_rpc::types::Id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selected {
    Current(i64),
    List(HashSet<i64>),
}

impl Selected {
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Current(_) => 1,
            Self::List(set) => set.len(),
        }
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::List(set) if set.is_empty())
    }
}

#[derive(Debug)]
pub enum SelectedIntoIter {
    One(Once<i64>),
    Many(IntoIter<i64>),
}

impl Iterator for SelectedIntoIter {
    type Item = i64;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::One(it) => it.next(),
            Self::Many(it) => it.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::One(it) => it.size_hint(),
            Self::Many(it) => it.size_hint(),
        }
    }
}

impl ExactSizeIterator for SelectedIntoIter {}

impl IntoIterator for Selected {
    type Item = i64;
    type IntoIter = SelectedIntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Current(id) => SelectedIntoIter::One(std::iter::once(id)),
            Self::List(set) => SelectedIntoIter::Many(set.into_iter()),
        }
    }
}

impl<S> From<Selected> for HashSet<i64, S>
where
    S: BuildHasher + Default,
{
    fn from(value: Selected) -> Self {
        value.into_iter().collect()
    }
}

impl From<Selected> for Vec<i64> {
    fn from(value: Selected) -> Self {
        value.into_iter().collect()
    }
}

impl From<Selected> for Vec<Id> {
    fn from(value: Selected) -> Self {
        value.into_iter().map(Id::Id).collect()
    }
}
