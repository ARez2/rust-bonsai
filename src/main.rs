use std::{time::{Duration, Instant}};
use bonsai::BonsaiTree;
use crossterm::{execute, terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen}, cursor, style::{Color, self, Stylize}, event::{poll, read, Event, KeyCode, KeyModifiers}};
use rand::{Rng, SeedableRng, rngs::ThreadRng};
use rand_chacha::ChaCha8Rng;
use simple_simplex::NoiseConfig;
use clap::Parser;


#[derive(Parser, Debug, Clone, Copy)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Starting with of the trunk
    #[clap(short, long, value_parser, default_value_t = 0)]
    width: u8,
    /// How many milliseconds need to pass between steps.
    #[clap(short, long, value_parser, default_value_t = 100)]
    time_scale: u64,
    /// Seed of the tree. Same seeds produce same trees
    #[clap(short, long, value_parser, default_value_t = 0)]
    seed: u64,
}


fn main() {
    // Setup program
    let args = Args::parse();
    let mut stdout = std::io::stdout();

    let mut initial_rng = rand::thread_rng();
    let mut tree = new_tree(args, &mut initial_rng);
    crossterm::execute!(std::io::stdout(), EnterAlternateScreen).unwrap();
    //crossterm::execute!(std::io::stdout(), Clear(ClearType::All)).unwrap();

    // Main loop
    let mut last_step = Instant::now();
    loop {
        let time_since_last_step = last_step.elapsed();
        if time_since_last_step > Duration::from_millis(args.time_scale) {
            tree.step();
            last_step = Instant::now();
        }
        
        if poll(Duration::from_millis(10)).unwrap() {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            match read().unwrap() {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Esc => break,
                        KeyCode::Char('r') => {
                            crossterm::execute!(std::io::stdout(), Clear(ClearType::All)).unwrap();
                            tree = new_tree(args, &mut initial_rng)
                        },
                        _ => println!("{:?}", event),
                    }
                },
                // Event::FocusGained => println!("FocusGained"),
                // Event::FocusLost => println!("FocusLost"),
                // Event::Mouse(event) => println!("{:?}", event),
                // #[cfg(feature = "bracketed-paste")]
                // Event::Paste(data) => println!("Pasted {:?}", data),
                // Event::Resize(width, height) => println!("New size {}x{}", width, height),
                _ => (),
            }
        } else {
            // Timeout expired and no `Event` is available 
        }
    }

    // Clean up
    crossterm::execute!(std::io::stdout(), LeaveAlternateScreen).unwrap();
    execute!(tree.stdout, cursor::MoveTo(0, 0)).unwrap();
}


pub fn new_tree(args: Args, initial_rng: &mut ThreadRng) -> BonsaiTree {
    let mut stdout = std::io::stdout();

    let mut seed = args.seed;
    if seed == 0 {
        seed = initial_rng.gen_range(0..std::u64::MAX - 1);
    };
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let freq = 4.0;
    let noise = NoiseConfig::new(
        1, // Octaves
        freq, // X-Frequency
        freq, // Y-Frequency
        2., // Amplitude
        2.5, // Lacunarity
        0.7, // Gain
        (-2.0, 2.0), // range
        seed // seed
    );

    let (width, height) = crossterm::terminal::size().unwrap();
    let mut trunk_width = args.width;
    if trunk_width == 0 {
        trunk_width = rng.gen_range(5..15);
    };
    
    bonsai::BonsaiTree::new(
        noise,
        rng,
        seed,
        stdout,
        width as i16,
        height as i16,
        trunk_width
    )
}