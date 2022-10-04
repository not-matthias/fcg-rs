use crate::parser::HeaderWithContent;
use itertools::Itertools;
use petgraph::graph::NodeIndex;
use petgraph::prelude::EdgeRef;
use petgraph::{Direction, Graph};
use regex::{Captures, Regex};

#[derive(Debug)]
pub struct Card {
    pub(crate) front: String,
    pub(crate) back: String,
}

impl Card {
    fn header_context(graph: &Graph<HeaderWithContent, usize>, index: NodeIndex) -> Vec<String> {
        let mut context_data = Vec::new();
        let mut current_index = index;

        while let Some(edge) = graph
            .edges_directed(current_index, Direction::Incoming)
            .next()
        {
            current_index = edge.source();

            if let Some(context) = graph.node_weight(current_index).map(|h| h.header.clone()) {
                context_data.push(context.into());
            }
        }

        // We went from bottom up, so we have to reverse the vec to get the correct order.
        //
        context_data.reverse();

        context_data
    }

    fn convert_obsidian_links(back: String) -> String {
        // TODO: https://github.com/reuseman/flashcards-obsidian/blob/main/src/services/parser.ts#L253

        back
    }

    fn convert_image_links(back: String) -> String {
        // TODO: https://github.com/reuseman/flashcards-obsidian/blob/main/src/services/parser.ts#L265

        back
    }

    fn convert_audio_links(back: String) -> String {
        // TODO: https://github.com/reuseman/flashcards-obsidian/blob/main/src/services/parser.ts#L272

        back
    }

    fn convert_math(back: String) -> String {
        let result = back;

        let block_regex = Regex::new(r"(\$\$)(.*?)(\$\$)").unwrap();
        let inline_regex = Regex::new(r"(\$)(.*?)(\$)").unwrap();

        let result = block_regex.replace(&result, |caps: &Captures| format!("\\[{}\\]", &caps[2]));
        let result = inline_regex.replace(&result, |caps: &Captures| format!("\\({}\\)", &caps[2]));

        result.to_string()
    }

    pub fn new(graph: &Graph<HeaderWithContent, usize>, index: NodeIndex) -> Self {
        let node = graph.node_weight(index).unwrap();
        let context = Self::header_context(graph, index);

        // Remove the unneeded `#` from the context
        //
        let context = context
            .into_iter()
            .map(|c| c.replace("# ", "").replace("#", ""))
            .join(" > ");

        let front = node.header.replace('#', "").trim().to_string();
        let front = if context.is_empty() {
            front
        } else {
            format!("{} > {}", context, front)
        };

        // Convert to html
        //
        let back = node.content.join("\n").trim().to_string();
        let back = Self::convert_obsidian_links(back);
        let back = Self::convert_image_links(back);
        let back = Self::convert_audio_links(back);
        let back = Self::convert_math(back);
        let back = comrak::markdown_to_html(&back, &comrak::ComrakOptions::default());

        Self { front, back }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
