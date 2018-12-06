//! See `core/src/subgraph/subgraphs.graphql` for corresponding graphql schema.

use std::collections::HashMap;

use graph::data::subgraph::schema::{SUBGRAPHS_ID, SUBGRAPH_ENTITY_TYPENAME};
use graph::data::subgraph::{Mapping, Source};
use graph::prelude::*;

pub struct SubgraphDefinitionStore<S> {
    store: Arc<S>,
}

impl<S> Clone for SubgraphDefinitionStore<S> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
        }
    }
}

impl<S> SubgraphDefinitionStore<S>
where
    S: Store,
{
    pub fn new(store: Arc<S>) -> Self {
        Self { store }
    }

    fn write(
        &self,
        entity_typename: impl Into<String>,
        entity_id: impl Into<String>,
        data: impl Into<Entity>,
    ) -> Result<(), Error> {
        self.store.apply_set_operation(
            EntityOperation::Set {
                key: EntityKey {
                    subgraph_id: SUBGRAPHS_ID.clone(),
                    entity_type: entity_typename.into(),
                    entity_id: entity_id.into(),
                },
                data: data.into(),
            },
            "none".to_owned(),
        )
    }

    pub fn write_subgraph(&self, manifest: SubgraphManifest, created_at: u64) -> Result<(), Error> {
        let id = manifest.id.clone();

        let mut entity = HashMap::new();

        entity.insert("id".to_owned(), id.to_string().into());
        entity.insert("createdAt".to_owned(), created_at.into());

        let manifest_id = format!("{}-manifest", id);
        self.write_manifest(manifest, manifest_id.clone())?;
        entity.insert("manifest".to_owned(), manifest_id.into());

        self.write(SUBGRAPH_ENTITY_TYPENAME, id.to_string(), entity)
    }

    pub fn write_manifest(&self, manifest: SubgraphManifest, id: String) -> Result<(), Error> {
        let mut entity = HashMap::new();

        entity.insert("id".to_owned(), id.clone().into());
        entity.insert("specVersion".to_owned(), manifest.spec_version.into());
        entity.insert("description".to_owned(), manifest.description.into());
        entity.insert("repository".to_owned(), manifest.repository.into());
        entity.insert(
            "schema".to_owned(),
            manifest.schema.document.to_string().into(),
        );

        let mut data_sources: Vec<Value> = vec![];
        for (i, data_source) in manifest.data_sources.into_iter().enumerate() {
            let data_source_id = format!("{}-data-source-{}", id, i);
            self.write_data_source(data_source, data_source_id.clone())?;
            data_sources.push(data_source_id.into());
        }
        entity.insert("dataSources".to_owned(), data_sources.into());

        self.write("SubgraphManifest", id, entity)
    }

    pub fn write_data_source(&self, data_source: DataSource, id: String) -> Result<(), Error> {
        let mut entity = HashMap::new();

        entity.insert("id".to_owned(), id.clone().into());
        entity.insert("kind".to_owned(), data_source.kind.into());
        entity.insert("network".to_owned(), data_source.network.into());
        entity.insert("name".to_owned(), data_source.name.into());

        let source_id = format!("{}-source", id);
        self.write_contract_source(data_source.source.clone(), source_id.clone())?;
        entity.insert("source".to_owned(), source_id.into());

        let mapping_id = format!("{}-mapping", id);
        self.write_contract_mapping(data_source.mapping, mapping_id.clone())?;
        entity.insert("mapping".to_owned(), mapping_id.into());

        self.write("EthereumContractDataSource", id, entity)
    }

    pub fn write_contract_source(&self, source: Source, id: String) -> Result<(), Error> {
        let mut entity = HashMap::new();

        entity.insert("id".to_owned(), id.clone().into());
        entity.insert("address".to_owned(), source.address.into());
        entity.insert("abi".to_owned(), source.abi.into());

        self.write("EthereumContractSource", id, entity)
    }

    pub fn write_contract_mapping(&self, mapping: Mapping, id: String) -> Result<(), Error> {
        let mut entity = HashMap::new();

        entity.insert("id".to_owned(), id.clone().into());
        entity.insert("kind".to_owned(), mapping.kind.into());
        entity.insert("apiVersion".to_owned(), mapping.api_version.into());
        entity.insert("language".to_owned(), mapping.language.into());
        entity.insert("file".to_owned(), mapping.link.link.into());

        let mut abis: Vec<Value> = vec![];
        for (i, abi) in mapping.abis.into_iter().enumerate() {
            let abi_id = format!("{}-abi-{}", id, i);
            self.write_mapping_abi(abi, abi_id.clone())?;
            abis.push(abi_id.into());
        }
        entity.insert("abis".to_owned(), abis.into());

        let entities: Vec<Value> = mapping.entities.into_iter().map(Value::from).collect();
        entity.insert("entities".to_owned(), entities.into());

        let mut event_handlers: Vec<Value> = Vec::new();
        for (i, event_handler) in mapping.event_handlers.into_iter().enumerate() {
            let event_handler_id = format!("{}-event-handler-{}", id, i);
            self.write_mapping_event_handler(event_handler, event_handler_id.clone())?;
            event_handlers.push(event_handler_id.into())
        }
        entity.insert("eventHandlers".to_owned(), event_handlers.into());

        self.write("EthereumContractMapping", id, entity)
    }

    pub fn write_mapping_abi(&self, abi: MappingABI, id: String) -> Result<(), Error> {
        let mut entity = HashMap::new();

        entity.insert("id".to_owned(), id.clone().into());
        entity.insert("name".to_owned(), abi.name.into());
        entity.insert("file".to_owned(), abi.link.link.into());

        self.write("EthereumContractAbi", id, entity)
    }

    pub fn write_mapping_event_handler(
        &self,
        handler: MappingEventHandler,
        id: String,
    ) -> Result<(), Error> {
        let mut entity = HashMap::new();

        entity.insert("id".to_owned(), id.clone().into());
        entity.insert("event".to_owned(), handler.event.into());
        entity.insert("handler".to_owned(), handler.handler.into());

        self.write("EthereumContractEventHandler", id, entity)
    }
}
