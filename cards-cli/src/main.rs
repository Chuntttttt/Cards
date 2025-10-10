use cards_core::CardWriter;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Turn directories of images into printable pdfs of card sheets")]
struct Args {
    #[arg(
        short = 'c',
        long,
        help = "Path to the folder containing the card images"
    )]
    cards_path: String,

    #[arg(
        short = 'o',
        long,
        default_value = "cards.pdf",
        help = "Path and filename for the output pdf"
    )]
    output: String,

    #[arg(
        short = 's',
        long,
        default_value_t = 3,
        help = "The number of sides in the grid (ex: 3 would produce a 3x3 grid of cards)"
    )]
    sides: usize,

    #[arg(short = 'v', long, help = "Log actions taken at each step")]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    let writer = CardWriter::new(args.cards_path, args.sides);
    writer.create_pdf(&args.output)?;

    Ok(())
}
