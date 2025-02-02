use std::{env, str::FromStr};

use alloy::{
    eips::BlockId,
    network::Ethereum,
    providers::{
        fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller},
        Identity, Provider, ProviderBuilder, RootProvider, WsConnect,
    },
    pubsub::SubscriptionStream,
    rpc::types::{Block, BlockTransactionsKind, Header},
    transports::http::reqwest::Url,
};

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
    pub ws_provider: WsProvider,
    pub http_provider: HttpProvider,
}

impl Providers {
    pub async fn new() -> Self {
        return Providers {
            ws_provider: Providers::get_ws_provider().await,
            http_provider: Providers::get_http_provider(),
        };
    }

    async fn get_ws_provider() -> WsProvider {
        let ws_url = env::var("WS_URL").unwrap();
        let ws_connect = WsConnect::new(ws_url);
        return ProviderBuilder::new().on_ws(ws_connect).await.unwrap();
    }

    fn get_http_provider() -> HttpProvider {
        let http_url = env::var("HTTP_URL").unwrap();
        let url: Url = Url::from_str(&http_url).unwrap();
        return ProviderBuilder::new().on_http(url);
    }

    pub async fn get_stream(&self) -> SubscriptionStream<Header> {
        let subscription = self.ws_provider.subscribe_blocks().await.unwrap();
        return subscription.into_stream();
    }

    pub async fn get_block(&self, block_number: u64) -> Block {
        return self
            .http_provider
            .get_block(BlockId::number(block_number), BlockTransactionsKind::Full)
            .await
            .unwrap()
            .unwrap();
    }
}
