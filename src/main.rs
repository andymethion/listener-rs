use futures_util::StreamExt;
use providers::Providers;
use token::TokensHandler;

mod providers;
mod token;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let providers = Providers::new().await;
    let mut stream = providers.get_stream().await;

    let tokens_handler = TokensHandler::new(&providers);

    while let Some(header) = stream.next().await {
        let block = providers.get_block(header.number).await;
        let transactions = block.transactions.into_transactions();
        tokens_handler.handle(transactions).await;
        println!("New Block Found {}", block.header.number);
    }
}
