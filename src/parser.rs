//! Parses the markdown files to find flashcards.
//!
//! This parser does the following:
//! - Extract deck name from front matter
//! -



use itertools::Itertools;
use petgraph::graph::NodeIndex;
use petgraph::dot::Dot;
use petgraph::{Directed, Direction, Graph};
use petgraph::prelude::{EdgeRef};

/// The struct representing the different markdown headers.
///
/// Allow nested headers for context aware mode:
/// ```
/// Header(1) { Content, Header(2) { Content, Header(3) { Content } }, Header(2) { Content } }
/// ```
#[derive(Debug)]
pub struct Header<'a> {
    /// The header level.
    level: usize,

    /// The content of the header. This will be the back of the card.
    content: Vec<&'a str>,

    /// The parent of the current heading.
    parent: Option<&'a Header<'a>>,
}

pub struct Parser<'a> {
    text: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    /// Tries to find a parent index for a heading.
    ///
    /// Example: We need to find the correct parent for the heading 3 and heading 2.
    /// ```
    /// ## Heading 2
    /// ### Heading 3
    /// #### Heading 4
    /// ### Heading 3
    /// ## Heading 2
    /// ```
    fn find_parent_index(graph: &Graph<Header, usize>, previous_index: NodeIndex, current_level: usize) -> Option<NodeIndex> {
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
        while let Some(edge) = graph.edges_directed(current_index, Direction::Incoming).last() {
            if let Some(index) = is_parent_correct(edge.source()) {
                return Some(index);
            }

            current_index = edge.source();
        }

        None
    }

    /// Parses the markdown document and returns a graph of headers.
    pub fn parse(&mut self) -> Graph<Header, usize> {
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
                    if line.contains("#") {
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
            log::trace!("Header: {:?}", header);
            log::trace!("Level: {:?}", level);

            let content: Vec<_> = self
                .text
                .lines()
                .take(next_index)
                .skip(index)
                .skip(1)
                .collect();
            log::trace!("Content: {:?}", content);

            // Create the header
            //
            let header = Header {
                level,
                content,
                parent: None,
            };

            if previous_index.is_some() && previous_level.is_some() {
                // Check if the new header is a sub-header
                // ```
                //       previous
                //     /         \
                //  HEADER     previous
                // ```
                if header.level > previous_level.unwrap()
                {
                    let node_index = graph.add_node(header);

                    if let Some(parent_index) = Self::find_parent_index(&graph, previous_index.unwrap(), level) {
                        graph.add_edge(parent_index, node_index, level);
                        log::warn!("Didn't create edge")
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
                else if header.level <= previous_level.unwrap()
                {
                    let node_index = graph.add_node(header);

                    if let Some(parent_index) = Self::find_parent_index(&graph, previous_index.unwrap(), level) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;
    use simple_logger::SimpleLogger;

    #[test]
    fn parse_test_1() {
        SimpleLogger::new()
            .with_level(LevelFilter::Trace)
            .init()
            .unwrap();

        let mut parser = Parser::new(include_str!("../data/test_graph.md"));
        let headers = parser.parse();

        println!("{:?}", headers);
    }
}
