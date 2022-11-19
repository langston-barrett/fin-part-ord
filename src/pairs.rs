use crate::r#trait::FinPartOrd;

#[derive(Clone, Debug)]
struct Pair<T> {
    lo: T,
    hi: T,
}

impl<T> Pair<T> {
    fn new(lo: T, hi: T) -> Self {
        Pair { lo, hi }
    }
}

/// A datatype for representing finite partial orders.
///
/// Stores partial orders as a vector of pairs. Doesn't store pairs that can
/// be deduced by reflexivity or transitivity. Maintains the invariant that it
/// always represents a valid partial order, specifically that the set of pairs
/// obeys antisymmetry. Provides poor computational complexity bounds but low
/// memory usage. Only requires that the contained type is `Eq`.
///
/// Not suitable for production use.
#[derive(Clone, Debug)]
pub struct PairPartOrd<T>
where
    T: Eq,
{
    pairs: Vec<Pair<T>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AntisymmetryError;

impl<T: Eq> FinPartOrd<T> for PairPartOrd<T>
where
    T: Clone,
{
    type Error = AntisymmetryError;

    #[must_use]
    fn empty() -> Self {
        let fpo = PairPartOrd { pairs: Vec::new() };
        debug_assert!(fpo.valid());
        fpo
    }

    fn add(mut self, lo: T, hi: T) -> Result<Self, Self::Error> {
        if lo == hi {
            return Ok(self);
        }
        self.pairs.push(Pair::new(lo, hi));
        if !self.valid() {
            return Err(AntisymmetryError);
        }
        Ok(self)
    }

    fn lt(&self, lo: &T, hi: &T) -> Result<bool, Self::Error> {
        for p in &self.pairs {
            if &p.lo == lo {
                if &p.hi == hi {
                    return Ok(true);
                }
                // DFS
                if let Ok(b) = self.lt(&p.hi, hi) {
                    if b {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }
}

impl<T: Eq> PairPartOrd<T>
where
    T: Clone,
{
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// For debugging only, return value should be considered unstable.
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.pairs.reserve(additional);
    }

    fn check(&self) -> Result<bool, AntisymmetryError> {
        for p in &self.pairs {
            if p.lo == p.hi {
                return Ok(false);
            }
            if self.lt(&p.lo, &p.lo)? || self.lt(&p.hi, &p.hi)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn valid(&self) -> bool {
        self.check().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use quickcheck::{quickcheck, Arbitrary, Gen};

    impl Arbitrary for PairPartOrd<u8> {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut ppo = PairPartOrd::empty();
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
            for i in 0..self.pairs.len() {
                let mut pairs = self.pairs.clone();
                pairs.remove(i);
                iters.push(PairPartOrd { pairs });
            }
            Box::new(iters.into_iter())
        }
    }

    #[test]
    fn empty_valid() {
        let mut empty = PairPartOrd::<()>::empty();
        assert!(empty.valid());
        empty.reserve(8);
        assert!(empty.valid());
    }

    #[test]
    fn push_unit_valid() {
        let unit = ();
        let mut ppo = PairPartOrd::empty();
        ppo = ppo.add(&unit, &unit).unwrap();
        assert!(ppo.valid());
        ppo = ppo.add(&unit, &unit).unwrap();
        assert!(ppo.valid());
    }

    #[test]
    fn strings() {
        let mut ppo = PairPartOrd::empty();
        ppo = ppo.add("x".to_string(), "y".to_string()).unwrap();
        ppo = ppo.add("y".to_string(), "z".to_string()).unwrap();
        assert!(ppo.le(&"x".to_string(), &"z".to_string()).unwrap());
    }

    quickcheck! {
        fn antisymmetric_two(x: u8, y: u8) -> bool {
            if x == y {
                return true;
            }
            let mut ppo = PairPartOrd::empty();
            ppo = ppo.add(&x, &y).unwrap();
            assert!(ppo.valid());
            ppo.add(&y, &x).is_err()
        }

        fn transitive_three(x: u8, y: u8, z: u8) -> bool {
            if x == z {
                return true;
            }
            let mut ppo = PairPartOrd::empty();
            ppo = ppo.add(x, y).unwrap();
            ppo = ppo.add(y, z).unwrap();
            assert!(ppo.valid());
            ppo.le(&x, &z).unwrap()
        }

        // TODO: Stack overflow :-(
        // fn add_le(ppo: PairPartOrd<u8>, x: u8, y: u8) -> bool {
        //     match ppo.add(x, y) {
        //         Err(_) => true,
        //         Ok(ppo) => {
        //             ppo.le(&x, &y).unwrap() && (!ppo.le(&y, &x).unwrap() || x == y)
        //         }
        //     }
        // }


        fn reflexive(ppo: PairPartOrd<u8>, x: u8) -> bool {
            ppo.le(&x, &x).unwrap()
        }

        fn antisymmetric(ppo: PairPartOrd<u8>, x: u8, y: u8) -> bool {
            if ppo.le(&x, &y).unwrap() && ppo.le(&y, &x).unwrap() {
                x == y
            } else {
                true
            }
        }

        fn transitive(ppo: PairPartOrd<u8>, x: u8, y: u8, z: u8) -> bool {
            if ppo.le(&x, &y).unwrap() && ppo.le(&y, &z).unwrap() {
                ppo.le(&x, &z).unwrap()
            } else {
                true
            }
        }
    }
}
