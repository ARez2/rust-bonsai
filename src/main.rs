use std::{u64::MAX, time::{Duration, Instant}, io::{Cursor, Write}};
use bonsai::{BonsaiTree, Writer};
use crossterm::{execute, terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen}, cursor, style::{Color, self, Stylize}, event::{poll, read, Event, KeyCode, KeyModifiers, EnableMouseCapture}};
use rand::{Rng, SeedableRng, rngs::ThreadRng};
use rand_chacha::ChaCha8Rng;
use simple_simplex::NoiseConfig;
use clap::Parser;


#[derive(Parser, Debug, Clone, Copy)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Starting with of the trunk
    #[clap(short, long, value_parser, default_value_t = 0)]
    width: usize,
    /// How fast the tree will grow
    #[clap(short, long, value_parser, default_value_t = 100)]
    time_scale: u64,
    /// Seed of the tree. Same seeds produce same trees
    #[clap(short, long, value_parser, default_value_t = 0)]
    seed: u64,
}


fn main() {
    let args = Args::parse();
    let buffer = Cursor::new(Vec::<u8>::new());
    let mut tree = grow_bonsai(args, buffer.clone());
    
    //execute!(std::io::stdout(), Clear(ClearType::All)).unwrap();
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen).unwrap();
    crossterm::terminal::enable_raw_mode().unwrap();
    execute!(stdout, EnableMouseCapture).unwrap();

    let mut last_step = Instant::now();
    loop {
        let time_since_last_step = last_step.elapsed();
        
        
        if poll(Duration::from_millis(0)).unwrap() {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            match read().unwrap() {
                // Event::FocusGained => println!("FocusGained"),
                // Event::FocusLost => println!("FocusLost"),
                Event::Key(event) => {
                    if event.modifiers.contains(KeyModifiers::CONTROL) && event.code == KeyCode::Char('c') {
                        break;
                    }
                    match event.code {
                        KeyCode::Esc => break,
                        KeyCode::Char('r') => {
                            crossterm::execute!(std::io::stdout(), Clear(ClearType::All)).unwrap();
                            tree = grow_bonsai(args, buffer.clone());
                        },
                        _ => println!("{:?}", event),
                    }
                },
                // Event::Mouse(event) => println!("{:?}", event),
                // #[cfg(feature = "bracketed-paste")]
                // Event::Paste(data) => println!("Pasted {:?}", data),
                // Event::Resize(width, height) => println!("New size {}x{}", width, height),
                _ => (),
            }
        } else {
            
        }
        //crossterm::execute!(std::io::stdout(), Clear(ClearType::All)).unwrap();
        if time_since_last_step > Duration::from_millis(args.time_scale) {
            tree.step();
            last_step = Instant::now();
        }
    }
    crossterm::terminal::disable_raw_mode().unwrap();
    crossterm::execute!(stdout, LeaveAlternateScreen).unwrap();
}


/// Sets up the growth of a new bonsai tree
fn grow_bonsai(args: Args, stdout: Writer) -> BonsaiTree {
    let mut rng = rand::thread_rng();
    //let mut stdout = std::io::stdout();

    let mut seed = args.seed;
    if seed == 0 {
        seed = rng.gen_range(0..MAX - 1);
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

    BonsaiTree::new(
        noise,
        rng,
        stdout,
        width as i16,
        height as i16,
        trunk_width
    )
}