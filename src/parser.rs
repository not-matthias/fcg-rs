use itertools::Itertools;
use petgraph::graph::NodeIndex;
use petgraph::prelude::EdgeRef;
use petgraph::{Direction, Graph};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct FileHeader {
    #[serde(rename = "cards-deck")]
    pub cards_deck: String,
}

impl Default for FileHeader {
    fn default() -> Self {
        Self {
            cards_deck: "default".into(),
        }
    }
}

/// The struct representing a markdown header with content.
#[derive(Debug)]
pub struct HeaderWithContent {
    /// The header level.
    pub level: usize,

    /// The header of the block.
    pub header: String,

    /// The content of the header. This will be the back of the card.
    pub content: Vec<String>,
}

pub struct Parser {
    text: String,
}

impl Parser {
    pub fn new(text: String) -> Self {
        Self { text }
    }

    /// Tries to find a parent index for a heading.
    fn find_parent_index(
        graph: &Graph<HeaderWithContent, usize>,
        previous_index: NodeIndex,
        current_level: usize,
    ) -> Option<NodeIndex> {
        // We need to do the following:
        // - Check node at previous_index if it's already at the level above
        // - Go up all parents until we find the

        // Check if the parent is already correct
        //
        let is_parent_correct = |index| {
            if let Some(weight) = graph.node_weight(index) {
                if weight.level < current_level {
                    return Some(index);
                }
            }

            None
        };

        if let Some(index) = is_parent_correct(previous_index) {
            return Some(index);
        }

        // Find the correct parent by going up the chain
        //
        let mut current_index = previous_index;
        while let Some(edge) = graph
            .edges_directed(current_index, Direction::Incoming)
            .last()
        {
            current_index = edge.source();

            if let Some(index) = is_parent_correct(current_index) {
                return Some(index);
            }
        }

        None
    }

    fn parse_markdown(&mut self) -> Graph<HeaderWithContent, usize> {
        let mut graph = Graph::new();
        let mut previous_index: Option<NodeIndex> = None;
        let mut previous_level: Option<usize> = None;

        // Calculate where all the headers start
        //
        let header_indices = {
            let mut header_indices: Vec<_> = self
                .text
                .lines()
                .enumerate()
                .filter_map(|(index, line)| {
                    if line.contains('#') {
                        Some(index)
                    } else {
                        None
                    }
                })
                .collect();

            // We just say that the last line is also a heading so the last tuple has a end.
            //
            header_indices.push(self.text.lines().count());

            header_indices
        };

        // Iterate over the headers
        //
        for (index, next_index) in header_indices.into_iter().tuple_windows() {
            let header = self.text.lines().nth(index).unwrap_or_default();
            let level = header.chars().filter(|c| c.eq(&'#')).count();
            log::trace!("Header {:?} with level {:?}", header, level);

            let content = self
                .text
                .lines()
                .take(next_index)
                .skip(index)
                .filter(|s| !s.is_empty())
                .skip(1) // Skip header
                .collect::<Vec<_>>();
            if content.is_empty() {
                continue;
            }
            log::trace!("Content: {:?}", content);

            // Create the header
            //
            let header = HeaderWithContent {
                level,
                header: header.into(),
                content: content.into_iter().map(|s| s.to_string()).collect(),
            };

            if previous_index.is_some() && previous_level.is_some() {
                // Check if the new header is a sub-header
                // ```
                //       previous
                //     /         \
                //  HEADER     previous
                // ```
                if header.level > previous_level.unwrap() {
                    let node_index = graph.add_node(header);

                    if let Some(parent_index) =
                        Self::find_parent_index(&graph, previous_index.unwrap(), level)
                    {
                        graph.add_edge(parent_index, node_index, level);
                    }

                    previous_index = Some(node_index);
                    previous_level = Some(level);
                }
                // Check if the new header is an upper or equal header
                // ```
                //          HEADER
                //       /         \
                //  previous     previous
                // ```
                else if header.level <= previous_level.unwrap() {
                    let node_index = graph.add_node(header);

                    if let Some(parent_index) =
                        Self::find_parent_index(&graph, previous_index.unwrap(), level)
                    {
                        graph.add_edge(parent_index, node_index, level);
                    }

                    previous_index = Some(node_index);
                    previous_level = Some(level);
                }
            } else {
                previous_index = Some(graph.add_node(header));
                previous_level = Some(level);
            }
        }

        // std::fs::write("out.dot", format!("{:?}", Dot::with_config(&graph, &[])));

        graph
    }

    fn parse_yaml(&mut self) -> FileHeader {
        // The only acceptable input is this:
        // ```
        // ---
        // cards-deck: test
        // ```
        //
        // So we have to find the last `---`:
        //
        let minus_indices: Vec<_> = self
            .text
            .lines()
            .enumerate()
            .filter_map(|(index, line)| {
                if line.contains("---") {
                    Some(index)
                } else {
                    None
                }
            })
            .collect();

        if let Some(last_row) = minus_indices.last() {
            serde_yaml::from_str(&*self.text.lines().take(*last_row).join("\n")).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    /// Parses and returns the header and markdown content as graph.
    pub fn parse(&mut self) -> (FileHeader, Graph<HeaderWithContent, usize>) {
        (self.parse_yaml(), self.parse_markdown())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_markdown() {
        let content = include_str!("../data/test-markdown.md").to_string();
        let mut parser = Parser::new(content);
        let content = parser.parse_markdown();

        insta::assert_debug_snapshot!(content);
    }

    #[test]
    fn parse_yaml() {
        let content = "---\ncards-deck: deck-name\n---".to_string();
        let mut parser = Parser::new(content);
        let content = parser.parse_yaml();

        insta::assert_debug_snapshot!(content);
    }
}
