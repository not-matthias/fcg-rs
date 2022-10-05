use once_cell::sync::OnceCell;
use std::path::PathBuf;

use deck::Deck;
use structopt::StructOpt;

pub mod card;
pub mod deck;
pub mod parser;

pub static RESOURCES_PATH: OnceCell<PathBuf> = OnceCell::new();

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Options {
    /// Path to the resources folder where the images are stored.
    #[structopt(short, long, parse(from_os_str))]
    resources: Option<PathBuf>,

    /// Path to the markdown file to convert.
    #[structopt(short, long, parse(from_os_str))]
    file: PathBuf,

    /// Path to the output directory. If not specified, the file will be saved to the current directory.
    #[structopt(short, long = "out_dir", parse(from_os_str))]
    output_dir: Option<PathBuf>,
}

fn main() {
    env_logger::init();

    let options = Options::from_args();

    // Set resources path
    //
    let resources = options.resources.unwrap_or_else(|| PathBuf::from("."));
    if !resources.exists() {
        log::error!("Resources path doesn't exist");
        return;
    }
    RESOURCES_PATH.set(resources).unwrap();
    log::info!("Using resources path: {:?}", RESOURCES_PATH.get().unwrap());

    // Convert file to deck
    //
    let file = std::fs::read_to_string(&options.file).unwrap();
    let deck = Deck::new(&file);

    let output_dir: PathBuf = options.output_dir.unwrap_or_else(|| PathBuf::from("."));
    let output_file = output_dir.join(format!("{}.apkg", deck.name));

    deck.save(output_file.to_str().unwrap());
}
