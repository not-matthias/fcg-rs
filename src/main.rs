use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::path::PathBuf;

use deck::Deck;
use structopt::StructOpt;
use walkdir::WalkDir;

pub mod card;
pub mod deck;
pub mod error;
pub mod parser;

pub static RESOURCES_PATH: OnceCell<PathBuf> = OnceCell::new();

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Options {
    /// Path to the resources folder where the images are stored.
    #[structopt(short, long, parse(from_os_str))]
    resources: Option<PathBuf>,

    /// Path to the markdown file to convert.
    #[structopt(long, parse(from_os_str))]
    file: Option<PathBuf>,

    /// Path to the folder of which all files will be converted.
    #[structopt(long, parse(from_os_str))]
    folder: Option<PathBuf>,

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

    // Set input folder path
    //
    let folder = options.folder.unwrap_or_else(|| PathBuf::from("."));
    log::info!("Fetching files from: {:?}", folder);

    // Iterate over all markdown files in the folder
    //
    let mut decks = HashMap::<String, Deck>::new();
    for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
        let name = entry.file_name().to_str().unwrap_or_default();

        // CHeck if we should only convert a single file
        if let Some(file) = options.file.as_ref() {
            if name != file.file_name().unwrap().to_str().unwrap() {
                continue;
            }
        }

        // Check if the file is a markdown file which can be converted
        if name.ends_with(".md") {
            let content = std::fs::read_to_string(entry.path()).unwrap();
            let deck = match Deck::new(&content) {
                Ok(deck) => deck,
                Err(error) => {
                    log::warn!("Failed to create anki deck: {:?} ({:?})", name, error);
                    continue;
                }
            };

            if decks.contains_key(&deck.name) {
                decks.get_mut(&deck.name).unwrap().combine(deck);
            } else {
                decks.insert(deck.name.clone(), deck);
            }
        }
    }

    // Save the decks to disk
    //
    let output_dir: PathBuf = options.output_dir.unwrap_or_else(|| PathBuf::from("."));

    for (name, deck) in decks {
        let output_file = output_dir.join(format!("{}.apkg", name));
        deck.save(output_file.to_str().unwrap());
    }
}
