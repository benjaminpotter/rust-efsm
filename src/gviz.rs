use crate::machine::Machine;
use num::Bounded;
use std::fmt;

pub struct GvGraph {
    nodes: Vec<GvNode>,
    edges: Vec<GvEdge>,
}

impl GvGraph {
    fn new() -> Self {
        GvGraph {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

struct GvNode {
    label: String,
    peripheries: u8,
}

struct GvEdge {
    label: String,
    head: String,
    tail: String,
}

impl From<GvGraph> for String {
    fn from(graph: GvGraph) -> Self {
        let mut spec = String::new();

        // Begin a new graph definition.
        spec.push_str("digraph machine {\n");
        spec.push_str("graph [center=true pad=.5];\n");
        spec.push_str("rankdir=LR;\n");

        for node in graph.nodes {
            spec.push_str(&format!(
                "{}[shape=circle,peripheries={}];\n",
                node.label, node.peripheries
            ));
        }

        for edge in graph.edges {
            spec.push_str(&format!(
                "{} -> {} [label=<{}>];\n",
                edge.head, edge.tail, edge.label
            ));
        }

        // Close the graph definition block.
        spec.push_str("}\n");

        spec
    }
}

impl<D, I, U> From<Machine<D, I, U>> for GvGraph
where
    D: fmt::Display + Bounded + Copy,
    U: fmt::Display,
{
    fn from(machine: Machine<D, I, U>) -> Self {
        let mut gv = GvGraph::new();

        for (location, transitions) in machine.get_locations() {
            // Double line for accepting states.
            let peripheries = match machine.get_accepting().contains(location) {
                true => 2,
                false => 1,
            };

            // Each state gets a GvNode.
            gv.nodes.push(GvNode {
                label: location.clone(),
                peripheries,
            });

            // Each transition gets a GvEdge.
            for t in transitions {
                gv.edges.push(GvEdge {
                    label: format!("{}<br/>{}", t.update, t.bound),

                    // TODO: We can avoid clone by referencing the machine's original copy.
                    // TODO: This requires that the machine outlives the graph.
                    // TODO: That requirement seems logical, and may be the best option.
                    // TODO: Further thought is required.
                    head: location.clone(),
                    tail: t.to_location.clone(),
                });
            }
        }

        gv
    }
}
