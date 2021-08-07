
use crate::units::RadPS;

// Interface for anything generating values on a per-sample basis.
// XXX deprecated
pub trait Gen {
    fn advance(&mut self) -> bool;
    fn gen(&self) -> f64;
    fn set_freq(&mut self, freq: impl Into<RadPS>);

    fn cost(&self) -> usize; // XXX prob shouldnt be here
}
