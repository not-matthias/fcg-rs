use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::{card::Card, parser::Parser};
use anki::{Field, Model, Note, Template};
use genanki_rs as anki;

pub struct Deck {
    pub name: String,
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn new(file: &str) -> Self {
        let mut parser = Parser::new(file.into());

        let header = parser.parse_yaml();
        let graph = parser.parse_markdown();

        let cards = graph
            .node_indices()
            .map(|index| Card::new(&graph, index))
            .collect();

        Self {
            name: header.cards_deck,
            cards,
        }
    }
}

impl Deck {
    fn str_to_id(string: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        string.hash(&mut hasher);
        hasher.finish()
    }

    fn card_model(card: &Card) -> Model {
        // let id = Self::str_to_id(&card.front) as usize;
        let template = Template::new("Card")
            .qfmt("{{Front}}")
            .afmt("{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}");

        Model::new(
            42,
            "Model",
            vec![Field::new("Front"), Field::new("Back")],
            vec![template],
        )
        .css(include_str!("card.css"))
    }

    /// Saves the current deck to disk.
    pub fn save(self, file: &str) {
        let mut deck = anki::Deck::new(
            Self::str_to_id(&self.name) as usize,
            &self.name,
            "No description available",
        );

        // Add all the notes
        //
        let notes = self.cards.iter().map(|card| {
            let model = Self::card_model(card);
            Note::new(model, vec![&card.front, &card.back]).unwrap()
        });
        for note in notes {
            deck.add_note(note);
        }

        deck.write_to_file(file).unwrap();
    }
}
