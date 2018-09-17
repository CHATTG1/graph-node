use failure::Error;
use futures::prelude::*;
use futures::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use graph::prelude::{
    BlockStream as BlockStreamTrait, BlockStreamBuilder as BlockStreamBuilderTrait,
    BlockStreamController as BlockStreamControllerTrait, EthereumBlock, *,
};
use graph::web3::types::{Block, Log, Transaction, H256};

/// Internal messages between the block stream controller and the block stream.
enum ControlMessage {
    Advance { block_hash: H256 },
}

pub struct BlockStream {}

impl BlockStream {
    pub fn new<C>(network: String, subgraph: String, chain_updates: C) -> Self
    where
        C: ChainHeadUpdateListener,
    {
        // TODO: Implement block stream algorithm whenever there is a chain update

        BlockStream {}
    }
}

impl BlockStreamTrait for BlockStream {}

impl Stream for BlockStream {
    type Item = EthereumBlock;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(Async::Ready(None))
    }
}

impl EventConsumer<ControlMessage> for BlockStream {
    fn event_sink(&self) -> Box<Sink<SinkItem = ControlMessage, SinkError = ()> + Send> {
        unimplemented!();
    }
}

pub struct BlockStreamController {
    sink: Sender<ControlMessage>,
    stream: Option<Receiver<ControlMessage>>,
}

impl BlockStreamController {
    pub fn new() -> Self {
        let (sink, stream) = channel(100);

        BlockStreamController {
            sink,
            stream: Some(stream),
        }
    }
}

impl EventProducer<ControlMessage> for BlockStreamController {
    fn take_event_stream(
        &mut self,
    ) -> Option<Box<Stream<Item = ControlMessage, Error = ()> + Send>> {
        self.stream
            .take()
            .map(|s| Box::new(s) as Box<Stream<Item = ControlMessage, Error = ()> + Send>)
    }
}

impl BlockStreamControllerTrait for BlockStreamController {
    fn advance(&self, block_hash: H256) -> Box<Future<Item = (), Error = ()> + Send> {
        Box::new(
            self.sink
                .clone()
                .send(ControlMessage::Advance { block_hash })
                .map(|_| ())
                .map_err(|_| ()),
        )
    }
}

pub struct BlockStreamBuilder<S, E> {
    store: Arc<Mutex<S>>,
    ethereum: Arc<Mutex<E>>,
    network: String,
}

impl<S, E> Clone for BlockStreamBuilder<S, E> {
    fn clone(&self) -> Self {
        BlockStreamBuilder {
            store: self.store.clone(),
            ethereum: self.ethereum.clone(),
            network: self.network.clone(),
        }
    }
}

impl<S, E> BlockStreamBuilder<S, E>
where
    S: ChainStore,
    E: EthereumAdapter,
{
    pub fn new(store: Arc<Mutex<S>>, ethereum: Arc<Mutex<E>>, network: String) -> Self {
        BlockStreamBuilder {
            store,
            ethereum,
            network,
        }
    }
}

impl<S, E> BlockStreamBuilderTrait for BlockStreamBuilder<S, E>
where
    S: ChainStore,
    E: EthereumAdapter,
{
    type Stream = BlockStream;
    type StreamController = BlockStreamController;

    fn from_subgraph(&self, manifest: &SubgraphManifest) -> (Self::Stream, Self::StreamController) {
        // Create chain update listener for the network used at the moment.
        //
        // NOTE: We only support a single network at this point, this is why
        // we're just picking the one that was passed in to the block stream
        // builder at the moment
        let chain_update_listener = self
            .store
            .lock()
            .unwrap()
            .chain_head_updates(self.network.as_str());

        // Create block stream controller
        let mut stream_controller = BlockStreamController::new();

        // Create the actual network- and subgraph-specific block stream
        let block_stream = BlockStream::new(
            self.network.clone(),
            manifest.id.clone(),
            chain_update_listener,
        );

        // Forward control messages from the stream controller to the block stream
        tokio::spawn(
            stream_controller
                .take_event_stream()
                .unwrap()
                .forward(block_stream.event_sink().sink_map_err(|_| ()))
                .and_then(|_| Ok(())),
        );

        (block_stream, stream_controller)
    }
}