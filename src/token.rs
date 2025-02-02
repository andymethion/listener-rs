use std::vec::IntoIter;

use alloy::{
    primitives::{Address, FixedBytes, U256},
    rpc::types::Transaction,
    sol,
};
use alloy_provider::ext::TraceApi;
use alloy_rpc_types_trace::parity::{LocalizedTransactionTrace, TraceOutput};
use tokio::join;

use crate::providers::Providers;

sol!(
    #[sol(rpc)]
    Erc20Abi,
    "src/abi/Erc20Abi.json",
);

struct Token {
    author: Address,
    address: Address,
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: U256,
}

pub struct TokensHandler<'a> {
    providers: &'a Providers,
}

impl<'a> TokensHandler<'a> {
    pub fn new(providers: &'a Providers) -> Self {
        return Self { providers };
    }

    async fn get_localized_transaction_traces(
        &self,
        transaction_hash: FixedBytes<32>,
    ) -> Vec<LocalizedTransactionTrace> {
        let localized_transaction_traces = self
            .providers
            .http_provider
            .trace_transaction(transaction_hash)
            .await;

        match localized_transaction_traces {
            Ok(localized_transaction_traces) => return localized_transaction_traces,
            _ => return Vec::new(),
        };
    }

    fn get_contract_addresses(
        &self,
        localized_transaction_traces: Vec<LocalizedTransactionTrace>,
    ) -> Vec<Address> {
        let mut contract_addresses = Vec::new();

        for localized_transaction_trace in localized_transaction_traces {
            let trace_output = match localized_transaction_trace.trace.result {
                Some(trace_output) => trace_output,
                _ => continue,
            };

            match trace_output {
                TraceOutput::Create(create_output) => {
                    contract_addresses.push(create_output.address)
                }
                _ => {}
            };
        }

        return contract_addresses;
    }

    async fn get_tokens(
        &self,
        transaction_from: Address,
        contract_addresses: Vec<Address>,
    ) -> Vec<Token> {
        let mut tokens = Vec::new();

        for contract_address in contract_addresses {
            let contract = Erc20Abi::new(contract_address, self.providers.http_provider.clone());

            let (name, symbol, decimals, total_supply) = join!(
                async { contract.name().call().await },
                async { contract.symbol().call().await },
                async { contract.decimals().call().await },
                async { contract.totalSupply().call().await },
            );

            if name.is_err() || symbol.is_err() || decimals.is_err() || total_supply.is_err() {
                continue;
            }

            let name = name.unwrap()._0;
            let symbol = symbol.unwrap()._0;
            let decimals = decimals.unwrap()._0;
            let total_supply = total_supply.unwrap()._0;

            tokens.push(Token {
                author: transaction_from,
                address: contract_address,
                name,
                symbol,
                decimals,
                total_supply,
            });
        }

        return tokens;
    }

    fn print_tokens(&self, tokens: Vec<Token>) {
        for token in tokens {
            println!("");
            println!("New Token Found");
            println!("Author: {}", token.author);
            println!("Address: {}", token.address);
            println!("Name: {}", token.name);
            println!("Symbol: {}", token.symbol);
            println!("Decimals: {}", token.decimals);
            println!("Total Supply: {}", token.total_supply);
            println!("");
        }
    }

    pub async fn handle(&self, transactions: IntoIter<Transaction>) {
        for transaction in transactions {
            let localized_transaction_traces = self
                .get_localized_transaction_traces(transaction.info().hash.unwrap())
                .await;

            let contract_addresses = self.get_contract_addresses(localized_transaction_traces);

            let tokens = self.get_tokens(transaction.from, contract_addresses).await;

            self.print_tokens(tokens);
        }
    }
}
