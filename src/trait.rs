/// Trait for finite partial orders.
///
/// Laws:
///
/// - Reflexivity: `self.le(x, x) == true`
/// - Antisymmetry: `self.le(x, y) && self.le(y, x) ==> x == y`
/// - Transitivity: `self.le(x, y) && self.le(y, z) ==> self.le(x, z)`
/// - Compatibility: `self.le(x, y) <==> x == y || self.lt(x, y)`
/// - Add/less: `self.add(x, y).le(x, y) == true`
pub trait FinPartOrd<T>
where
    Self: Sized,
{
    type Error;

    #[must_use]
    fn empty() -> Self;

    fn add(self, lo: T, hi: T) -> Result<Self, Self::Error>;

    /// Check if one element is less than another.
    ///
    /// May return `true` when `lo == hi`, even if that element hasn't been
    /// explicitly added.
    fn lt(&self, lo: &T, hi: &T) -> Result<bool, Self::Error>;

    /// May return `true` when `lo == hi`, even if that element hasn't been
    /// explicitly added.
    fn le(&self, lo: &T, hi: &T) -> Result<bool, Self::Error>
    where
        T: PartialEq,
    {
        if lo == hi {
            return Ok(true);
        }
        self.lt(lo, hi)
    }
}
