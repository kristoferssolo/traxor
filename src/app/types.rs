use std::collections::HashSet;

use transmission_rpc::types::Id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selected {
    Current(i64),
    List(HashSet<i64>),
}

impl Into<HashSet<i64>> for Selected {
    fn into(self) -> HashSet<i64> {
        match self {
            Selected::Current(id) => vec![id].into_iter().collect(),
            Selected::List(ids) => ids,
        }
    }
}

impl Into<Vec<i64>> for Selected {
    fn into(self) -> Vec<i64> {
        match self {
            Selected::Current(id) => vec![id],
            Selected::List(ids) => ids.into_iter().collect(),
        }
    }
}

impl Into<Vec<Id>> for Selected {
    fn into(self) -> Vec<Id> {
        match self {
            Selected::Current(id) => vec![Id::Id(id)],
            Selected::List(ids) => ids.into_iter().map(Id::Id).collect(),
        }
    }
}
