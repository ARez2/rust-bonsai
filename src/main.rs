use std::{u64::MAX, time::Duration};
use crossterm::{execute, terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen}, cursor, style::{Color, self, Stylize}, event::{poll, read, Event, KeyCode, KeyModifiers}};
use rand::{Rng, SeedableRng, rngs::ThreadRng};
use rand_chacha::ChaCha8Rng;
use simple_simplex::NoiseConfig;
use clap::Parser;


#[derive(Parser, Debug, Clone, Copy)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Starting with of the trunk
    #[clap(short, long, value_parser, default_value_t = 0)]
    width: u8,
    /// How fast the tree will grow
    #[clap(short, long, value_parser, default_value_t = 1.0)]
    time_scale: f32,
    /// Seed of the tree. Same seeds produce same trees
    #[clap(short, long, value_parser, default_value_t = 0)]
    seed: u64,
}


fn main() {
    let args = Args::parse();
    
    //execute!(stdout, Clear(ClearType::All)).unwrap();
    crossterm::execute!(std::io::stdout(), EnterAlternateScreen).unwrap();
    grow_bonsai(args);
    loop {
        if poll(Duration::from_millis(500)).unwrap() {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            match read().unwrap() {
                // Event::FocusGained => println!("FocusGained"),
                // Event::FocusLost => println!("FocusLost"),
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Esc => break,
                        KeyCode::Char('r') => grow_bonsai(args),
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
            // Timeout expired and no `Event` is available 
        }
    }
    crossterm::execute!(std::io::stdout(), LeaveAlternateScreen).unwrap();
}


fn grow_bonsai(args: Args) {
    let stdout = std::io::stdout();
    let mut rng = rand::thread_rng();

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
    let mut tree = bonsai::BonsaiTree::new(
        noise,
        rng,
        stdout,
        width as i16,
        height as i16,
        trunk_width
    );
    tree.time_multiplier = args.time_scale;
    execute!(tree.stdout,
        Clear(ClearType::All),
        cursor::MoveTo(1, height - 2),
        style::PrintStyledContent(format!("Seed: {}", seed).with(Color::DarkGrey))
    ).unwrap();
    tree.grow();
    execute!(tree.stdout, cursor::MoveTo(0, 0)).unwrap();
}