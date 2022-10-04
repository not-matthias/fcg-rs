use once_cell::sync::OnceCell;
use std::path::PathBuf;

use deck::Deck;
use structopt::StructOpt;
use walkdir::WalkDir;

pub mod card;
pub mod deck;
pub mod parser;

pub static RESOURCES_PATH: OnceCell<PathBuf> = OnceCell::new();

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Options {
    /// Path to the resources folder where the images are stored.
    #[structopt(short, long, parse(from_os_str))]
    resources: PathBuf,

    /// Path to the markdown file to convert.
    #[structopt(short, long, parse(from_os_str))]
    file: PathBuf,
}

fn get_decks() -> Vec<Deck> {
    let mut decks: Vec<Deck> = Vec::new();

    for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        let name = entry.file_name().to_str().unwrap_or_default();

        if name.ends_with(".md") {
            let content = std::fs::read_to_string(entry.path()).unwrap();

            decks.push(Deck::new(&content))
        }
    }

    decks
}

fn main() {
    env_logger::init();

    let options = Options::from_args();

    // Set resources path
    if !options.resources.exists() {
        log::error!("Resources path doesn't exist");
        return;
    }
    RESOURCES_PATH.set(options.resources).unwrap();

    // Convert file to deck
    //
    let file = std::fs::read_to_string(&options.file).unwrap();
    let deck = Deck::new(&file);
    deck.save("obsidian-test.apkg");
}
