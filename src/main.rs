mod freq;

fn repeat(ch: char, n: i64) {
    for _ in 0..n {
        print!("{}", ch);
    }
}

fn plot1(x: f64) {
    let center = 40;
    let width = 74;
    let mut off = (((x * width as f64) / 2.0) as i64) + center;
    if off < 1 { off = 1; }
    if off > 78 { off = 78; }
    if off == center {
        repeat(' ', center-1);
        repeat('*', 1);
        repeat('\n', 1);
    } else if off < center {
        repeat(' ', off-1);
        repeat('*', 1);
        repeat('-', center-off-1);
        repeat('|', 1);
        repeat('\n', 1);
    } else {
        repeat(' ', center-1);
        repeat('|', 1);
        repeat('-', off-center-1);
        repeat('*', 1);
        repeat('\n', 1);
    }
}

fn plot(gen: &mut freq::HarmonicGenerator) {
    let DECIMATE = 44100 / 30;
    for n in 0 .. freq::SAMPLE_RATE as i64 {
        if n % DECIMATE == 0 {
            plot1(gen.gen());
        }
        gen.advance();
    }
    println!("Cost {:?}", gen.cost());
    println!("");
}

fn main() {
    plot(&mut freq::HarmonicGenerator::new_sine(2.0));
    plot(&mut freq::HarmonicGenerator::new_triangle(2.0, 10));
    plot(&mut freq::HarmonicGenerator::new_saw_up(2.0, 10));
    plot(&mut freq::HarmonicGenerator::new_square(2.0, 10));

    // cost: 2
    let mut gen = freq::HarmonicGenerator::new_saw_up(10000.0, 40);
    debug_assert!(gen.cost() == 2);

    // verify phase continuity
    gen.set_freq(0.5);
    plot(&mut gen);
    gen.set_sine();
    plot(&mut gen);
}
