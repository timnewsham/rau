
use crate::module;

// Interface for anything generating values on a per-sample basis.
pub trait Gen {
    fn advance(&mut self);
    fn gen(&self) -> f64;

    // XXX prob shouldnt be here
    fn cost(&self) -> usize;
}
                                                                                
impl<T> module::Module for T where T: Gen {
    fn get_terminals(&self) -> (Vec<module::TerminalDescr>, Vec<module::TerminalDescr>) {
        (vec![], 
         vec!["out".to_string()])
    }                                                                           
                                                                                
    fn get_output(&self, idx: usize) -> Option<f64> {                           
        if idx == 0 {
            Some(self.gen())
        } else {
            unreachable!();
        }
    }                                                                           
                                                                                
    fn set_input(&mut self, idx: usize, value: f64) {                           
        unreachable!();
    }                                                                           
                                                                                
    fn advance(&mut self) {                                                     
        self.advance();
    }                                                                           
}
