use crate::parser::HeaderWithContent;
use itertools::Itertools;
use latex2mathml::{latex_to_mathml, DisplayStyle};
use petgraph::graph::NodeIndex;
use petgraph::prelude::EdgeRef;
use petgraph::{Direction, Graph};
use regex::Regex;

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
        // TODO: https://github.com/reuseman/flashcards-obsidian/blob/main/src/services/parser.ts#L276
        // TODO:  Avoid compiling the same regex in a loop

        let mut result = back;

        let block_regex = Regex::new(r"(\$\$)(.*?)(\$\$)").unwrap();
        // if let Some(captures) = block_regex.captures(&result) {
        //     // result = latex2mathml::replace(&result).unwrap();
        //
        //     for capture in captures.iter() {
        //         if let Some(capture) = capture {
        //             println!("{:?}", latex2mathml::latex_to_mathml(capture.as_str(), DisplayStyle::Block))
        //         }
        //     }
        // }
        if block_regex.is_match(&*result) {
            if let Ok(latex) = latex_to_mathml(&result, DisplayStyle::Block) {
                result = latex;
            } else {
                log::warn!("Couldn't convert latex ({:?})", result);
            }
        }

        let inline_regex = Regex::new(r"(\$)(.*?)(\$)").unwrap();
        // if let Some(captures) = inline_regex.captures(&result) {
        //     for capture in captures.iter() {
        //         if let Some(capture) = capture {
        //             let new_result = result.replace(capture.as_str(), &*latex2mathml::latex_to_mathml(capture.as_str(), DisplayStyle::Inline).unwrap());
        //             println!("{:?}", new_result);
        //         }
        //     }
        // }
        if inline_regex.is_match(&*result) {
            if let Ok(latex) = latex_to_mathml(&result, DisplayStyle::Inline) {
                result = latex;
            } else {
                log::warn!("Couldn't convert latex ({:?})", result);
            }
        }

        result
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
