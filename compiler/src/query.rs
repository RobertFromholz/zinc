//! A query is a pure function.
//!
//! A query should only depend on data collected or generated from other queries. A query invoked
//! with equal inputs should always produce the same result.
//!
//! Queries are executed by calling a `query::Context`. The context creates a `query::Handle`,
//! a unique object given to every query. The query uses the handle to call other queries. In turn,
//! the handle records every query invoked to generate a dependency graph.

use crate::dot;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// A context executes all queries.
///
/// The context records a dependency graph consisting of all queries.
///
/// The context cannot be accessed by queries. Instead, queries are given a handle to a context.
pub struct Context {
    queries: RefCell<HashMap<QueryId, QueryData>>,
}

struct QueryData {
    state: QueryState,
    dependencies: HashSet<QueryId>,
}

#[derive(Clone, Debug)]
enum QueryState {
    /// This query has been created but has not been computed.
    Created,
    /// This query is being computed.
    Computing,
    /// This query has already been computed.
    Computed(Rc<dyn Any>),
}

impl Context {
    pub fn new() -> Self {
        Self {
            queries: RefCell::new(HashMap::new()),
        }
    }

    pub fn execute<Q: Query>(&self, key: Q::Key) -> Q::Output {
        let handle = Handle {
            query: None,
            context: ParentHandle::Context(self),
        };
        handle.execute::<Q>(key)
    }
}

/// A handle used by queries to call other queries.
pub struct Handle<'ctx> {
    query: Option<QueryId>,
    context: ParentHandle<'ctx>,
}

// A handle needs to be able to be built on top of another handle.
// The root handle still needs an actual context.
enum ParentHandle<'ctx> {
    Context(&'ctx Context),
    Handle(&'ctx Handle<'ctx>),
}

impl Handle<'_> {
    pub fn execute<Q: Query>(&self, key: Q::Key) -> Q::Output {
        let context = self.context();
        let query = {
            let mut queries = context.queries.borrow_mut();
            let query = QueryId::new::<Q>(key);
            queries.entry(query.clone()).or_insert_with(|| QueryData {
                state: QueryState::Created,
                dependencies: HashSet::new(),
            });
            if let Some(parent_query) = &self.query {
                let parent = queries.get_mut(parent_query).unwrap();
                parent.dependencies.insert(query.clone());
            }
            let data = queries.get_mut(&query).unwrap();
            match &data.state {
                QueryState::Created => data.state = QueryState::Computing,
                QueryState::Computing => panic!("Cycle detected: {:?} -> {:?}", self.query, query),
                QueryState::Computed(value) => return value.downcast_ref::<Q::Output>().unwrap().clone(),
            }
            query
        };
        let key = query.key.as_any().downcast_ref::<Q::Key>().unwrap();
        let output = Q::execute(Handle {
            query: Some(query.clone()),
            context: ParentHandle::Handle(self),
        }, key);
        {
            let mut queries = context.queries.borrow_mut();
            queries.entry(query).and_modify(|data| {
                data.state = QueryState::Computed(Rc::new(output.clone()));
            });
        };
        output
    }

    fn context(&self) -> &Context {
        match self.context {
            ParentHandle::Context(context) => context,
            ParentHandle::Handle(handle) => handle.context(),
        }
    }
}

pub trait QueryKey: Any + Debug + 'static {
    fn as_any(&self) -> &dyn Any;

    fn dyn_eq(&self, other: &dyn QueryKey) -> bool;

    fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl<T: Any + PartialEq + Eq + Hash + Debug + 'static> QueryKey for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn QueryKey) -> bool {
        other.as_any()
            .downcast_ref::<T>()
            .is_some_and(|other| self == other)
    }

    fn dyn_hash(&self, mut state: &mut dyn Hasher) {
        TypeId::of::<T>().hash(&mut state);
        self.hash(&mut state);
    }
}

// A query. The same query invoked with the same arguments will always produce the same value.
pub trait Query: 'static {
    type Key: QueryKey;
    type Output: Clone;

    fn execute(handle: Handle<'_>, key: &Self::Key) -> Self::Output;
}

/// A `QueryId` identifies a specific query. It does not reference the actual query, and as such
/// can't be used to call the query. Instead, it is used to identify a query's dependencies.
///
/// A `QueryId` is guaranteed to exist in `Context::queries`.
#[derive(Clone)]
struct QueryId {
    type_id: TypeId,
    // We use the name of the type to help with debugging.
    // In the future, it could also be used when serializing and deserializing a query to disk.
    // Although this might be problematic since the name isn't guaranteed to be unique.
    name: &'static str,
    key: Rc<dyn QueryKey>,
}

impl QueryId {
    /// Create a new `QueryId` to identify a query of the given type.
    fn new<Q: Query>(key: Q::Key) -> Self {
        Self {
            type_id: TypeId::of::<Q>(),
            name: std::any::type_name::<Q>(),
            key: Rc::new(key),
        }
    }
}

impl PartialEq for QueryId {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id && self.key.dyn_eq(&*other.key)
    }
}

impl Eq for QueryId {}

impl Hash for QueryId {
    fn hash<H: Hasher>(&self, mut state: &mut H) {
        self.type_id.hash(state);
        self.key.dyn_hash(&mut state);
    }
}

impl fmt::Debug for QueryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({:?})", self.name, self.key)
    }
}

impl dot::Graph<QueryId, (QueryId, QueryId)> for Context {
    fn nodes(&self) -> Vec<QueryId> {
        self.queries.borrow()
            .keys()
            .cloned()
            .collect()
    }

    fn edges(&self) -> Vec<(QueryId, QueryId)> {
        self.queries.borrow().iter()
            .flat_map(|(query, data)| {
                data.dependencies.iter()
                    .map(|dependency| (query.clone(), dependency.clone()))
            }).collect()
    }
}

impl dot::Node for QueryId {
    fn id(&self) -> String {
        format!("{:?}", self)
    }
}

impl dot::Edge for (QueryId, QueryId) {
    fn left(&self) -> String {
        format!("{:?}", self.0)
    }

    fn right(&self) -> String {
        format!("{:?}", self.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct LiteralQuery;

    impl Query for LiteralQuery {
        type Key = usize;
        type Output = usize;

        fn execute(_handle: Handle, key: &usize) -> usize {
            *key
        }
    }

    struct AddQuery;

    impl Query for AddQuery {
        type Key = (usize, usize);
        type Output = usize;

        fn execute(handle: Handle<'_>, key: &Self::Key) -> Self::Output {
            let left = handle.execute::<LiteralQuery>(key.0);
            let right = handle.execute::<LiteralQuery>(key.1);
            left + right
        }
    }

    #[test]
    fn simple_literal_query() {
        let context = Context::new();
        let value = context.execute::<LiteralQuery>(0);
        assert_eq!(value, 0);
        {
            let queries = context.queries.borrow();
            assert_eq!(queries.len(), 1);
        }
    }

    #[test]
    fn query_with_dependency() {
        let context = Context::new();
        let value = context.execute::<AddQuery>((2, 3));
        assert_eq!(value, 5);
        {
            let queries = context.queries.borrow();
            // We have executed 3 unique queries.
            // The same query with a different key still counts as a different query.
            assert_eq!(queries.len(), 3);
            let query = QueryId::new::<AddQuery>((2, 3));
            let data = queries.get(&query).unwrap();
            assert!(matches!(data.state, QueryState::Computed(_)));
            assert_eq!(data.dependencies, HashSet::from([
                QueryId::new::<LiteralQuery>(2),
                QueryId::new::<LiteralQuery>(3),
            ]));
        }
        // context.draw_and_open_graph().unwrap();
    }
}
