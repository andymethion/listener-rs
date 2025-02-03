use std::vec::IntoIter;

use alloy::{
    primitives::{Address, FixedBytes, U256},
    rpc::types::Transaction,
    sol,
    transports::{RpcError, TransportErrorKind},
};
use alloy_provider::ext::TraceApi;
use alloy_rpc_types_trace::parity::{LocalizedTransactionTrace, TraceOutput};
use tokio::join;

use crate::providers::Providers;

sol!(
    #[sol(rpc)]
    Erc20Abi,
    "src/abis/Erc20Abi.json",
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
    ) -> Result<Vec<LocalizedTransactionTrace>, RpcError<TransportErrorKind>> {
        return self
            .providers
            .http_provider
            .trace_transaction(transaction_hash)
            .await;
    }

    fn get_contract_address(
        &self,
        localized_transaction_trace: LocalizedTransactionTrace,
    ) -> Option<Address> {
        let trace_output = match localized_transaction_trace.trace.result {
            Some(trace_output) => trace_output,
            _ => return None,
        };

        match trace_output {
            TraceOutput::Create(create_output) => return Some(create_output.address),
            _ => return None,
        };
    }

    async fn get_token(
        &self,
        transaction_from: Address,
        contract_address: Address,
    ) -> Option<Token> {
        let contract = Erc20Abi::new(contract_address, self.providers.http_provider.clone());

        let (name, symbol, decimals, total_supply) = join!(
            async { contract.name().call().await },
            async { contract.symbol().call().await },
            async { contract.decimals().call().await },
            async { contract.totalSupply().call().await },
        );

        let (name, symbol, decimals, total_supply) = match (name, symbol, decimals, total_supply) {
            (Ok(name), Ok(symbol), Ok(decimals), Ok(total_supply)) => {
                (name._0, symbol._0, decimals._0, total_supply._0)
            }
            _ => return None,
        };

        return Some(Token {
            author: transaction_from,
            address: contract_address,
            name,
            symbol,
            decimals,
            total_supply,
        });
    }

    fn print_token(&self, token: Token) {
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

    fn handle_token(&self, token: Token) {
        self.print_token(token);
    }

    async fn handle_contract_address(&self, transaction_from: Address, contract_address: Address) {
        let token = self.get_token(transaction_from, contract_address).await;

        let token = match token {
            Some(token) => token,
            _ => return,
        };

        self.handle_token(token);
    }

    async fn handle_localized_transaction_trace(
        &self,
        transaction_from: Address,
        localized_transaction_trace: LocalizedTransactionTrace,
    ) {
        let contract_address = self.get_contract_address(localized_transaction_trace);

        let contract_address = match contract_address {
            Some(contract_address) => contract_address,
            _ => return,
        };

        self.handle_contract_address(transaction_from, contract_address)
            .await;
    }

    async fn handle_transaction(&self, transaction: Transaction) {
        let localized_transaction_traces = self
            .get_localized_transaction_traces(transaction.info().hash.unwrap())
            .await;

        let localized_transaction_traces = match localized_transaction_traces {
            Ok(localized_transaction_traces) => localized_transaction_traces,
            _ => return,
        };

        for localized_transaction_trace in localized_transaction_traces {
            self.handle_localized_transaction_trace(transaction.from, localized_transaction_trace)
                .await;
        }
    }

    pub async fn handle(&self, transactions: IntoIter<Transaction>) {
        for transaction in transactions {
            self.handle_transaction(transaction).await;
        }
    }
}
