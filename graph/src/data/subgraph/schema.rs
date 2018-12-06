use super::SubgraphId;

/// ID of the subgraph of subgraphs.
lazy_static! {
    pub static ref SUBGRAPHS_ID: SubgraphId = SubgraphId::new("subgraphs").unwrap();
}

/// Type name of the root entity in the subgraph of subgraphs.
pub const SUBGRAPH_ENTITY_TYPENAME: &str = "Subgraph";
