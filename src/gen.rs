
// Interface for anything generating values on a per-sample basis.
pub trait Gen {
    fn advance(&mut self);
    fn gen(&self) -> f64;

    // XXX prob shouldnt be here
    fn cost(&self) -> usize;
}

