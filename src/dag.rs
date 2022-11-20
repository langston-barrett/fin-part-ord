///! DAG-based implementation of [`FinPartOrd`].
use std::collections::HashMap;
use std::hash::Hash;

use daggy::{Dag, NodeIndex, WouldCycle};
use petgraph::visit::Dfs;

use crate::r#trait::FinPartOrd;

/// A datatype for representing finite partial orders.
///
/// Stores partial order as a [`Dag`]. Doesn't store edges that can be deduced
/// by reflexivity or transitivity. Maintains the invariant that it always
/// represents a valid partial order, specifically that the set of edges obeys
/// antisymmetry, that is, forms a DAG.
#[derive(Clone, Debug)]
struct DagPartOrd<T> {
    dag: Dag<T, ()>,
    ids: HashMap<T, NodeIndex>,
}

impl<T> FinPartOrd<T> for DagPartOrd<T>
where
    T: Clone,
    T: Eq,
    T: Hash,
{
    type Error = WouldCycle<()>;

    #[must_use]
    fn empty() -> Self {
        DagPartOrd {
            dag: Dag::new(),
            ids: HashMap::new(),
        }
    }

    fn add(mut self, lo: T, hi: T) -> Result<Self, Self::Error> {
        if lo == hi {
            return Ok(self);
        }
        let lo_idx = match self.ids.get(&lo) {
            Some(lo_idx) => *lo_idx,
            None => {
                let id = self.dag.add_node(lo.clone());
                self.ids.insert(lo, id);
                id
            }
        };
        let hi_idx = match self.ids.get(&hi) {
            Some(hi_idx) => *hi_idx,
            None => {
                let id = self.dag.add_node(hi.clone());
                self.ids.insert(hi, id);
                id
            }
        };
        self.dag.add_edge(lo_idx, hi_idx, ())?;
        Ok(self)
    }

    fn lt(&self, lo: &T, hi: &T) -> Result<bool, Self::Error> {
        match (self.ids.get(lo), self.ids.get(hi)) {
            (Some(lo_idx), Some(hi_idx)) => {
                let mut dfs = Dfs::new(&self.dag, *lo_idx);
                while let Some(n) = dfs.next(&self.dag) {
                    if n == *hi_idx {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use quickcheck::{quickcheck, Arbitrary, Gen};

    impl Arbitrary for DagPartOrd<u8> {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut ppo = DagPartOrd::empty();
            let pairs = Vec::<(u8, u8)>::arbitrary(g);
            for (x, y) in pairs {
                if x <= y {
                    ppo = ppo.add(x, y).unwrap();
                } else {
                    ppo = ppo.add(y, x).unwrap();
                }
            }
            ppo
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            let mut iters = Vec::new();
            match self.dag.graph().node_indices().next() {
                Some(ix) => {
                    let mut new = self.clone();
                    new.dag.remove_node(ix);
                    iters.push(new);
                }
                None => (),
            }
            Box::new(iters.into_iter())
        }
    }

    #[test]
    fn empty() {
        let _ = DagPartOrd::<()>::empty();
    }

    #[test]
    fn push_unit() {
        let unit = ();
        DagPartOrd::empty().add(&unit, &unit).unwrap();
    }

    #[test]
    fn strings() {
        let mut ppo = DagPartOrd::empty();
        ppo = ppo.add("x".to_string(), "y".to_string()).unwrap();
        ppo = ppo.add("y".to_string(), "z".to_string()).unwrap();
        assert!(ppo.le(&"x".to_string(), &"z".to_string()).unwrap());
    }

    quickcheck! {
        fn antisymmetric_two(x: u8, y: u8) -> bool {
            if x == y {
                return true;
            }
            let mut ppo = DagPartOrd::empty();
            ppo = ppo.add(&x, &y).unwrap();
            ppo.add(&y, &x).is_err()
        }

        fn transitive_three(x: u8, y: u8, z: u8) -> bool {
            if x == z {
                return true;
            }
            let mut ppo = DagPartOrd::empty();
            ppo = ppo.add(x, y).unwrap();
            ppo = ppo.add(y, z).unwrap();
            ppo.le(&x, &z).unwrap()
        }

        fn add_le(ppo: DagPartOrd<u8>, x: u8, y: u8) -> bool {
            match ppo.add(x, y) {
                Err(_) => true,
                Ok(ppo) => {
                    ppo.le(&x, &y).unwrap() && (!ppo.le(&y, &x).unwrap() || x == y)
                }
            }
        }


        fn reflexive(ppo: DagPartOrd<u8>, x: u8) -> bool {
            ppo.le(&x, &x).unwrap()
        }

        fn antisymmetric(ppo: DagPartOrd<u8>, x: u8, y: u8) -> bool {
            if ppo.le(&x, &y).unwrap() && ppo.le(&y, &x).unwrap() {
                x == y
            } else {
                true
            }
        }

        fn transitive(ppo: DagPartOrd<u8>, x: u8, y: u8, z: u8) -> bool {
            if ppo.le(&x, &y).unwrap() && ppo.le(&y, &z).unwrap() {
                ppo.le(&x, &z).unwrap()
            } else {
                true
            }
        }
    }
}
