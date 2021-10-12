use std::path::PathBuf;
use genanki_rs::{Deck, Field, Model, Note, Template};
use crate::card::Card;

pub struct AnkiConfig {
    pub(crate) deck_name: String,
    pub(crate) deck_description: String
}

pub struct AnkiDeck {
    config: AnkiConfig,
    cards: Vec<Card>
}

impl AnkiDeck {
    fn create_model() -> Model {
       Model::new(
            1607392319,
            "Model",
            vec![Field::new("Question"), Field::new("Answer")],
            vec![Template::new("Card 1")
                .qfmt("{{Question}}")
                .afmt(r#"{{FrontSide}}<hr id="answer">{{Answer}}"#)],
        )
    }

    pub fn new(config: AnkiConfig, cards: Vec<Card>) -> Self {
        Self {
            config,
            cards
        }
    }

    /// Writes the deck with the cards to the file.
    pub fn write_to_file(&self, file: &str) {
        let model = Self::create_model();

        let notes: Vec<_> = self.cards.iter()
            .map(|card| {
                Note::new(model.clone(), vec![card.front.as_ref(), card.back.as_ref()]).unwrap()
            })
            .collect();

        let mut deck = Deck::new(2059400110, &*self.config.deck_name, &*self.config.deck_description);
        for note in notes {
            deck.add_note(note);
        }

        deck.write_to_file(file).unwrap();
    }

    /// Syncs the cards to the server.
    pub fn sync(&self) {
        todo!()
    }
}