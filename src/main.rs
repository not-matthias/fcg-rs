use crate::anki::{AnkiConfig, AnkiDeck};
use crate::card::Card;
use crate::parser::{HeaderWithContent, Parser};
use genanki_rs::{Deck, Field, Model, Note, Template};
use itertools::Itertools;
use log::LevelFilter;
use petgraph::graph::NodeIndex;
use petgraph::{Direction, Graph};
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::path::PathBuf;
use structopt::StructOpt;
use walkdir::WalkDir;

pub mod anki;
pub mod card;
pub mod config;
pub mod parser;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Options {
    /// Activate log output
    #[structopt(short, long)]
    debug: bool,
}

fn get_decks() -> HashMap<String, Vec<Card>> {
    let mut decks: HashMap<String, Vec<Card>> = HashMap::new();

    // Find a
    for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        let name = entry.file_name().to_str().unwrap_or_default();

        if name.ends_with(".md") {
            // Create cards
            //
            let text = std::fs::read_to_string(entry.path()).unwrap();
            let mut parser = Parser::new(text.into());
            let (header, graph) = parser.parse();
            let cards = graph
                .node_indices()
                .map(|index| Card::new(&graph, index))
                .collect();

            // Add cards to map
            //
            let deck_name = header.cards_deck;
            if decks.contains_key(&deck_name) {
                let deck = decks.get_mut(&deck_name).unwrap();

                deck.extend(cards);
            } else {
                decks.insert(deck_name, cards);
            }
        }
    }

    decks
}

fn main() {
    let options = Options::from_args();

    if options.debug {
        SimpleLogger::new()
            .with_level(LevelFilter::Info)
            .init()
            .unwrap();
    }

    // Convert to deck
    //

    for (deck_name, cards) in get_decks() {
        let deck = AnkiDeck::new(
            AnkiConfig {
                deck_name: deck_name.clone(),
                deck_description: "Description".to_string(),
            },
            cards,
        );
        deck.write_to_file(&format!("{}.apkg", deck_name));
    }
}
