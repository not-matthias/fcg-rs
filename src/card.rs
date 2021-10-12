use crate::parser::HeaderWithContent;
use itertools::Itertools;
use petgraph::data::DataMap;
use petgraph::graph::NodeIndex;
use petgraph::prelude::EdgeRef;
use petgraph::{Direction, Graph};
use std::borrow::Cow;

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

    pub fn new(graph: &Graph<HeaderWithContent, usize>, index: NodeIndex) -> Self {
        let node = graph.node_weight(index).unwrap();
        let context = Self::header_context(graph, index);

        // Remove the unneeded `#` from the context
        //
        let context = context
            .into_iter()
            .map(|c| c.replace("# ", "").replace("#", ""))
            .join(" > ");

        let back = node.content.join("\n").trim().to_string();
        let back = comrak::markdown_to_html(&back, &comrak::ComrakOptions::default());

        Self {
            front: format!("{} > {}", context, node.header.replace('#', "").trim()).into(),
            back: back,
        }
    }
}
