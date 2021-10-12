use crate::card::Card;
use genanki_rs::{Deck, Field, Model, Note, Template};
use std::path::PathBuf;

pub struct AnkiConfig {
    pub(crate) deck_name: String,
    pub(crate) deck_description: String,
}

pub struct AnkiDeck {
    config: AnkiConfig,
    cards: Vec<Card>,
}

impl AnkiDeck {
    fn create_model() -> Model {
        let css = ".card {\r\n font-family: arial;\r\n font-size: 20px;\r\n text-align: center;\r\n color: black;\r\n background-color: white;\r\n}\r\n\r\n.tag::before {\r\n\tcontent: \"#\";\r\n}\r\n\r\n.tag {\r\n  color: white;\r\n  background-color: #9F2BFF;\r\n  border: none;\r\n  font-size: 11px;\r\n  font-weight: bold;\r\n  padding: 1px 8px;\r\n  margin: 0px 3px;\r\n  text-align: center;\r\n  text-decoration: none;\r\n  cursor: pointer;\r\n  border-radius: 14px;\r\n  display: inline;\r\n  vertical-align: middle;\r\n}\r\n";
        Model::new_with_options(
            1607392319,
            "Model",
            vec![Field::new("Front"), Field::new("Back")],
            vec![Template::new("Card 1")
                .qfmt("{{Front}}")
                .afmt("{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}")],
            Some(css),
            None,
            None,
            None,
            None,
        )
    }

    pub fn new(config: AnkiConfig, cards: Vec<Card>) -> Self {
        Self { config, cards }
    }

    /// Writes the deck with the cards to the file.
    pub fn write_to_file(&self, file: &str) {
        let model = Self::create_model();

        let notes: Vec<_> = self
            .cards
            .iter()
            .map(|card| {
                Note::new(model.clone(), vec![card.front.as_ref(), card.back.as_ref()]).unwrap()
            })
            .collect();

        let mut deck = Deck::new(
            2059400110,
            &*self.config.deck_name,
            &*self.config.deck_description,
        );
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
