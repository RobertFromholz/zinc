//! A query-system to order computations.
//!
//! Queries are pure functions. Given a key, it computes and returns a value. Queries can, and
//! should, call other queries for intermediate computations. A query's key should be small and
//! inexpensive create or compare against. A query should, given the same key, always produce
//! the same result. Similarly, the result should be easy to clone, since every subsequent
//! call for the same query will return the same cached result.
//!
//! Depending on the context, a query is used to both denote the function and a specific combination
//! of a query function and a key.
//!
//! The system is organized by a `query::Context`. The context keeps track of what queries are being
//! or have already been computed.
//!
//! The context is used to call a query. In doing so, the context creates a `query::Handle` which
//! is given the query. The handle can be used to call other queries. The handle records what
//! queries are called to build a dependency graph for the query.
//!
//! Every combination of a query and its key is treated as a distinct computation and as such has a
//! potentially distinct set of dependencies.
//!
//! As of now, dependencies aren't necessarily required and are not used. We assume that
//! computations are stable for the duration of the program and will only change between two
//! program runs. We also don't currently cache computations to disk, which means we will always
//! need to recompute queries every time we run the program.

use crate::dot;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::fmt;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// A context used to compute and track queries.
///
/// The context builds a dependency graph for all queries.
///
/// The context cannot be accessed by queries. Instead, queries are given a `query::Handle`.
pub struct Context {
    queries: RefCell<HashMap<QueryId, QueryData>>,
}

struct QueryData {
    state: QueryState,
    dependencies: HashSet<QueryId>,
}

#[derive(Clone, Debug)]
enum QueryState {
    /// This query is being computed.
    ///
    /// If such a query is encountered whilst executing another query. The program has encountered
    /// a cycle and will panic.
    Computing,

    /// This query has already been computed.
    ///
    /// The result will be returned immediately and the query will not be computed again.
    Computed(Rc<dyn Any>),
}

impl Context {
    /// Create a new, empty, context.
    pub fn new() -> Self {
        Self {
            queries: RefCell::new(HashMap::new()),
        }
    }

    /// Compute a query.
    pub fn compute<Q: Query>(&self, key: Q::Key) -> Q::Output {
        let handle = Handle {
            query: None,
            context: self,
        };
        handle.compute::<Q>(key)
    }
}

/// A handle used to compute and track queries called from inside a query.
///
/// All queries called from a handle will be registered as dependencies to the current query.
pub struct Handle<'ctx> {
    query: Option<QueryId>,
    context: &'ctx Context,
}

impl Handle<'_> {
    pub fn compute<Q: Query>(&self, key: Q::Key) -> Q::Output {
        let query = {
            let mut queries = self.context.queries.borrow_mut();
            let query = QueryId::new::<Q>(key);

            // Register this query as a dependency to the parent query.
            // We technically haven't created the query yet, but it will either succeed or panic.
            // By doing this now, we can safely return later if the query has already been computed
            // without risking forgetting to register the dependency.
            if let Some(parent_query) = &self.query {
                // We assume the parent query exists in 'queries'.
                // Otherwise, 'queries' must be corrupted, and there isn't a good way for us to
                // recover.
                let parent = queries.get_mut(parent_query).unwrap();
                parent.dependencies.insert(query.clone());
            }

            match queries.entry(query.clone()) {
                Entry::Occupied(entry) => {
                    let data = entry.get();
                    match &data.state {
                        QueryState::Computing => {
                            panic!("Cycle detected: {:?} -> {:?}", self.query, query)
                        },
                        QueryState::Computed(value) => {
                            // Two query_id's can only ever be equivalent if they are of the same query.
                            // This means Q must be the same type, which means Q::Output must be the
                            // same type. This means the already computed value must be of the Q::Output
                            // type. If it isn't, we shouldn't try to recover.
                            return value.downcast_ref::<Q::Output>().unwrap().clone();
                        }
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(QueryData {
                        state: QueryState::Computing,
                        dependencies: HashSet::new(),
                    });
                }
            }

            query
        };
        // We have just created the 'query_id', so we know it is of the correct type.
        // We need to downcast it since we move the key into 'query_id'.
        // Ideally, we'd want to just be able to clone the key, but the key isn't required to be
        // cloneable. This is because requiring cloneable makes it more difficult to store the key
        // as a dynamic object since Clone isn't dyn compatible.
        let key = query.key.as_any().downcast_ref::<Q::Key>().unwrap();
        // Actually compute the query.
        let output = Q::compute(Handle {
            query: Some(query.clone()),
            context: self.context,
        }, key);
        {
            let mut queries = self.context.queries.borrow_mut();
            // The query must exist in 'queries' since we just created it.
            let query = queries.get_mut(&query).unwrap();
            // Cache the result.
            query.state = QueryState::Computed(Rc::new(output.clone()));
        };
        output
    }
}

/// An internal trait used by a query's key.
///
/// It is necessary since we can't use a dynamic object's PartialEq or Hash method. Instead, we need
/// to define our own methods that work with any dynamic key.
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

pub trait Query: 'static {
    /// The key used by this query.
    ///
    /// The key *must* implement `PartialEq`, `Eq`, `Hash` and `Debug`.
    type Key: QueryKey;
    /// The result of this query.
    type Output: Clone;

    /// Compute this query. Called internally by a `query::Handle`.
    ///
    /// To compute this query, either call `query::Context::compute` or `query::Handle::compute`
    /// respectively, depending on whether you want to compute this query from a global scope or
    /// from within another query.
    fn compute(handle: Handle<'_>, key: &Self::Key) -> Self::Output;
}

/// A `QueryId` uniquely identifies the combination of a query and a key.
///
/// A `QueryId` is guaranteed to exist in `Context::queries`.
#[derive(Clone)]
pub struct QueryId {
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

        fn compute(_handle: Handle, key: &usize) -> usize {
            *key
        }
    }

    struct AddQuery;

    impl Query for AddQuery {
        type Key = (usize, usize);
        type Output = usize;

        fn compute(handle: Handle, key: &(usize, usize)) -> usize {
            let left = handle.compute::<LiteralQuery>(key.0);
            let right = handle.compute::<LiteralQuery>(key.1);
            left + right
        }
    }

    struct FibonacciQuery;

    impl Query for FibonacciQuery {
        type Key = usize;
        type Output = usize;

        fn compute(handle: Handle, key: &usize) -> usize {
            match key {
                0 => 0,
                1 => 1,
                key => {
                    let left = handle.compute::<FibonacciQuery>(key - 1);
                    let right = handle.compute::<FibonacciQuery>(key - 2);
                    left + right
                }
            }
        }
    }

    #[test]
    fn simple_literal_query() {
        let context = Context::new();
        let value = context.compute::<LiteralQuery>(0);
        assert_eq!(value, 0);
        {
            let queries = context.queries.borrow();
            assert_eq!(queries.len(), 1);
        }
    }

    #[test]
    fn query_with_dependency() {
        let context = Context::new();
        let value = context.compute::<AddQuery>((2, 3));
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

    #[test]
    fn simple_query_with_caching() {
        let context = Context::new();
        let value = context.compute::<AddQuery>((2, 3));
        assert_eq!(value, 5);
        let value = context.compute::<AddQuery>((3, 4));
        assert_eq!(value, 7);
        assert_eq!(context.queries.borrow().len(), 5);
    }

    #[test]
    fn query_with_caching() {
        let context = Context::new();
        let value = context.compute::<FibonacciQuery>(5);
        assert_eq!(value, 5);
        assert_eq!(context.queries.borrow().len(), 6);
    }
}
