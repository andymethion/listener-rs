use std::{env, str::FromStr, vec::IntoIter};

use alloy::{
    network::Ethereum,
    primitives::{Address, TxHash, B256},
    pubsub::SubscriptionStream,
    rpc::types::{Block, BlockTransactionsKind, Filter, Header, Log, Transaction},
    transports::http::reqwest::Url,
};
use alloy_provider::{
    ext::TraceApi,
    fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller},
    Identity, Provider, ProviderBuilder, RootProvider, WsConnect,
};
use alloy_rpc_types_trace::parity::LocalizedTransactionTrace;

type WsProvider = FillProvider<
    JoinFill<
        Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    RootProvider,
    Ethereum,
>;

type HttpProvider = FillProvider<
    JoinFill<
        Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    RootProvider,
    Ethereum,
>;

pub struct Providers {
    ws_provider: WsProvider,
    pub http_provider: HttpProvider,
}

impl Providers {
    async fn get_ws_provider() -> WsProvider {
        let ws_url = env::var("WS_URL").unwrap();
        let ws_connect = WsConnect::new(ws_url);
        return ProviderBuilder::new().on_ws(ws_connect).await.unwrap();
    }

    fn get_http_provider() -> HttpProvider {
        let http_url = env::var("HTTP_URL").unwrap();
        let url = Url::from_str(&http_url).unwrap();
        return ProviderBuilder::new().on_http(url);
    }

    pub async fn new() -> Self {
        return Self {
            ws_provider: Self::get_ws_provider().await,
            http_provider: Self::get_http_provider(),
        };
    }

    pub async fn get_stream(&self) -> SubscriptionStream<Header> {
        let subscription = self.ws_provider.subscribe_blocks().await.unwrap();
        return subscription.into_stream();
    }

    async fn get_block(&self, block_number: u64) -> Block {
        return self
            .http_provider
            .get_block(block_number.into(), BlockTransactionsKind::Full)
            .await
            .unwrap()
            .unwrap();
    }

    pub async fn get_transactions(&self, block_number: u64) -> IntoIter<Transaction> {
        let block = self.get_block(block_number).await;
        return block.transactions.into_transactions();
    }

    pub async fn get_localized_transaction_traces(
        &self,
        transaction_hash: TxHash,
    ) -> Vec<LocalizedTransactionTrace> {
        return self
            .http_provider
            .trace_transaction(transaction_hash)
            .await
            .unwrap();
    }

    pub async fn get_logs(&self, block_hash: B256) -> Vec<Log> {
        let filter = Filter::new().at_block_hash(block_hash);
        return self.http_provider.get_logs(&filter).await.unwrap();
    }
}
