
mod ascii;
mod freq;
mod gen;

fn main() {
    ascii::plot(&mut gen::HarmonicGenerator::new_sine(2.0));
    ascii::plot(&mut gen::HarmonicGenerator::new_triangle(2.0, 10));
    ascii::plot(&mut gen::HarmonicGenerator::new_saw_up(2.0, 10));
    ascii::plot(&mut gen::HarmonicGenerator::new_square(2.0, 10));

    // cost: 2
    let mut gen = gen::HarmonicGenerator::new_saw_up(10000.0, 40);
    debug_assert!(gen.cost() == 2);

    // verify phase continuity
    gen.set_freq(0.5);
    ascii::plot(&mut gen);
    gen.set_sine();
    ascii::plot(&mut gen);
}
