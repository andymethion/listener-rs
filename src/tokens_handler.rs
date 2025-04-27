use std::vec::IntoIter;

use alloy::{
    primitives::{Address, U256},
    rpc::types::Transaction,
    sol,
};
use alloy_rpc_types_trace::parity::{LocalizedTransactionTrace, TraceOutput};
use tokio::join;

use crate::providers::Providers;

sol!(
    #[sol(rpc)]
    Erc20,
    "src/abis/Erc20Abi.json",
);

#[derive(Debug)]
pub struct Token {
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
        return TokensHandler { providers };
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

    async fn get_name(&self, address: Address) -> Option<String> {
        let contract = Erc20::new(address, self.providers.http_provider.clone());
        return contract.name().call().await.ok().map(|name| name._0);
    }

    async fn get_symbol(&self, address: Address) -> Option<String> {
        let contract = Erc20::new(address, self.providers.http_provider.clone());
        return contract.symbol().call().await.ok().map(|symbol| symbol._0);
    }

    async fn get_decimals(&self, address: Address) -> Option<u8> {
        let contract = Erc20::new(address, self.providers.http_provider.clone());
        return contract
            .decimals()
            .call()
            .await
            .ok()
            .map(|decimals| decimals._0);
    }

    async fn get_total_supply(&self, address: Address) -> Option<U256> {
        let contract = Erc20::new(address, self.providers.http_provider.clone());
        return contract
            .totalSupply()
            .call()
            .await
            .ok()
            .map(|total_supply| total_supply._0);
    }

    pub async fn get_token(&self, address: Address) -> Option<Token> {
        let (name, symbol, decimals, total_supply) = join!(
            self.get_name(address),
            self.get_symbol(address),
            self.get_decimals(address),
            self.get_total_supply(address),
        );

        let (name, symbol, decimals, total_supply) = match (name, symbol, decimals, total_supply) {
            (Some(name), Some(symbol), Some(decimals), Some(total_supply)) => {
                (name, symbol, decimals, total_supply)
            }
            _ => return None,
        };

        return Some(Token {
            address,
            name,
            symbol,
            decimals,
            total_supply,
        });
    }

    fn handle_token(&self, transaction_from: Address, token: Token) {
        println!();
        println!("New Token Handled");
        println!("Author: {}", transaction_from);
        println!("Address: {}", token.address);
        println!("Name: {}", token.name);
        println!("Symbol: {}", token.symbol);
        println!("Decimals: {}", token.decimals);
        println!("Total Supply: {}", token.total_supply);
        println!();
    }

    async fn handle_contract_address(&self, transaction_from: Address, contract_address: Address) {
        let token = self.get_token(contract_address).await;

        let token = match token {
            Some(token) => token,
            _ => return,
        };

        self.handle_token(transaction_from, token);
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
            .providers
            .get_localized_transaction_traces(transaction.info().hash.unwrap())
            .await;

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
