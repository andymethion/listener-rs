use futures_util::StreamExt;
use pools_handler::PoolsHandler;
use providers::Providers;
use swaps_handler::SwapsHandler;
use tokens_handler::TokensHandler;
use tokio::join;
use transfers_handler::TransfersHandler;

mod pools_handler;
mod providers;
mod swaps_handler;
mod tokens_handler;
mod transfers_handler;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let providers = Providers::new().await;
    let mut stream = providers.get_stream().await;

    let tokens_handler = TokensHandler::new(&providers);
    let transfers_handler = TransfersHandler::new();
    let pools_handler = PoolsHandler::new();
    let swaps_handler = SwapsHandler::new();

    while let Some(header) = stream.next().await {
        println!("Block Number {} Received", header.number);

        let (transactions, logs) = join!(
            providers.get_transactions(header.number),
            providers.get_logs(header.hash),
        );

        tokens_handler.handle(transactions.clone()).await;
        transfers_handler.handle(&logs);
        pools_handler.handle(&logs);
        swaps_handler.handle(&logs);

        println!("Transactions Len: {}", transactions.len());
        println!("Logs Len: {}", logs.len());
        println!("Block Number {} Handled", header.number);
    }
}
