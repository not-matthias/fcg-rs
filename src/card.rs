use base64::engine::general_purpose;
use base64::Engine;
use image::ImageOutputFormat;
use std::io::Cursor;
use std::path::PathBuf;

use crate::parser::CardGraph;
use crate::RESOURCES_PATH;
use itertools::Itertools;
use petgraph::graph::NodeIndex;
use petgraph::prelude::EdgeRef;
use petgraph::Direction;
use regex::{Captures, Regex};

// Credits to https://github.com/reuseman/flashcards-obsidian/blob/main/src/conf/regex.ts
lazy_static::lazy_static! {
    static ref OBSIDIAN_IMAGE_LINK : Regex = Regex::new(r#"!\[\[(.*\.(?:png|jpg|jpeg|gif|bmp|svg|tiff)).*?\]\]"#).unwrap();
    static ref MARKDOWN_IMAGE_LINK : Regex = Regex::new(r#"!\[\]\((.*\.(?:png|jpg|jpeg|gif|bmp|svg|tiff)).*?\)"#).unwrap();
    static ref LATEX_BLOCK_REGEX : Regex = Regex::new(r"(\$\$)(.*?)(\$\$)").unwrap();
    static ref LATEX_INLINE_REGEX : Regex = Regex::new(r"(\$)(.*?)(\$)").unwrap();
    static ref OBSIDIAN_LINK : Regex = Regex::new(r"(\[\[)(.*?)(\]\])").unwrap();
}

#[derive(Debug)]
pub struct Card {
    pub(crate) front: String,
    pub(crate) back: String,
}

impl Card {
    fn header_context(graph: &CardGraph, index: NodeIndex) -> Vec<String> {
        let mut context_data = Vec::new();
        let mut current_index = index;

        while let Some(edge) = graph
            .edges_directed(current_index, Direction::Incoming)
            .next()
        {
            current_index = edge.source();

            if let Some(context) = graph.node_weight(current_index).map(|h| h.header.clone()) {
                context_data.push(context);
            }
        }

        // We went from bottom up, so we have to reverse the vec to get the correct order.
        //
        context_data.reverse();

        context_data
    }

    fn image_to_base64(path: PathBuf) -> String {
        let image = image::open(path).unwrap();

        let mut data: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        image.write_to(&mut cursor, ImageOutputFormat::Png).unwrap();

        let base64 = general_purpose::STANDARD_NO_PAD.encode(data);
        format!("<img src='data:image/png;base64,{}'>", base64)
    }

    /// Converts Obsidian Links (e.g. `[[Term]]`) into normal text (e.g. `Term`).
    fn convert_obsidian_links(text: String) -> String {
        let result = OBSIDIAN_LINK.replace_all(&text, |caps: &Captures| (caps[2]).to_string());

        result.to_string()
    }

    fn convert_image_links(back: String) -> String {
        let replacer = |caps: &Captures| {
            let path = RESOURCES_PATH.get().unwrap().join(&caps[1]);
            if !path.exists() {
                log::warn!("{} not found.", path.display());
                return "Image not found".to_string();
            }

            Self::image_to_base64(path)
        };

        let result = back;
        let result = OBSIDIAN_IMAGE_LINK.replace_all(&result, replacer);
        let result = MARKDOWN_IMAGE_LINK.replace_all(&result, replacer);

        result.to_string()
    }

    fn convert_math(back: String) -> String {
        let result = back;
        let result =
            LATEX_BLOCK_REGEX.replace_all(&result, |caps: &Captures| format!("\\[{}\\]", &caps[2]));
        let result = LATEX_INLINE_REGEX
            .replace_all(&result, |caps: &Captures| format!("\\({}\\)", &caps[2]));

        result.to_string()
    }

    fn card_from_graph(graph: &CardGraph, index: NodeIndex) -> (String, String) {
        let node = graph.node_weight(index).unwrap();
        let context = Self::header_context(graph, index);

        // Remove the unneeded `#` from the context
        //
        let context = context
            .into_iter()
            .map(|c| c.replace("# ", "").replace('#', ""))
            .join(" > ");

        let front = node.header.replace('#', "").trim().to_string();
        let front = if context.is_empty() {
            front
        } else {
            format!("{} > {}", context, front)
        };

        let back = node.content.join("\n").trim().to_string();

        (front, back)
    }

    pub fn new(graph: &CardGraph, index: NodeIndex) -> Self {
        let (front, back) = Self::card_from_graph(graph, index);

        let front = Self::convert_obsidian_links(front);
        let front = Self::convert_image_links(front);
        let front = Self::convert_math(front);

        let back = Self::convert_obsidian_links(back);
        let back = Self::convert_image_links(back);
        let back = Self::convert_math(back);

        // let options = ComrakOptions {
        //     extension: ComrakExtensionOptions::default(),
        //     parse: ComrakParseOptions::default(),
        //     render: ComrakRenderOptions {
        //         unsafe_: true,
        //         ..ComrakRenderOptions::default()
        //     },
        // };
        // let back = comrak::markdown_to_html(&back, &options);

        Self { front, back }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_obsidian_link() {
        let input = "this is a [[term]].".to_string();
        let converted = Card::convert_obsidian_links(input);

        insta::assert_display_snapshot!(converted);
    }

    #[test]
    fn test_convert_obsidian_embeds() {
        RESOURCES_PATH.set(PathBuf::from("./data")).unwrap();

        let input = include_str!("../data/test-obsidian.md").to_string();
        let converted = Card::convert_image_links(input);

        insta::assert_display_snapshot!(converted);
    }

    #[test]
    fn test_convert_math() {
        let input = r#"$E = m c^2$ is the most famous equation derived by Einstein.
        In fact, this relation is a spacial case of the equation
        $$E = \sqrt{ m^2 c^4 + p^2 c^2 } ,$$
        which describes the relation between energy and momentum."#
            .to_string();
        let converted = Card::convert_math(input);

        insta::assert_display_snapshot!(converted);
    }

    #[test]
    fn test_asdf() {
        let input = r#"
        ### Wie kann man von $\mathbb R ^{\mathbb N}$ ausgehend einen Vektorraum konstruieren?"#
            .to_string();
        let converted = Card::convert_math(input);

        insta::assert_display_snapshot!(converted);
    }
}
