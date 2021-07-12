mod freq;

fn repeat(ch: char, n: i64) {
    for _ in 0..n {
        print!("{}", ch);
    }
}

fn plot(x: f64) {
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

fn plotGen(gen: &mut freq::AddGenerator) {
    for _ in 0 .. freq::SAMPLE_RATE as i64 {
        plot(gen.gen());
        gen.advance();
    }
}

fn mainx() {
    //let mut phase = freq::Freq::default();
    //let adv = freq::Freq::from_hz(440.0);
    //let adv = freq::Freq::from_hz(2.0);
    //let mut last = freq::Freq::default();
    //let mut accum = freq::PhaseAccum::from_hz(440.0);
    let mut accum = freq::PhaseAccum::from_hz(1.0);
    
    //println!("I got {:?} {:?}", phase, adv);
    for _ in 0 .. freq::SAMPLE_RATE as i64 {
        //println!("phase {:?}", phase);
        //last = phase;
        plot(accum.sin());
        accum.advance();
        //phase.advance(adv);
        //if phase < last { println!("cycle!"); }
    }
}

fn main() {
    plotGen(&mut freq::AddGenerator::new_sin(2.0));
    plotGen(&mut freq::AddGenerator::new_triangle(2.0, 5));
    plotGen(&mut freq::AddGenerator::new_saw(2.0, 5));
    plotGen(&mut freq::AddGenerator::new_square(2.0, 5));
}
