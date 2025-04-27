use std::{env, str::FromStr};

use alloy::{
    primitives::{Address, Log as PrimitivesLog, B256, U256},
    rpc::types::Log as TypesLog,
    sol,
    sol_types::SolEvent,
};

sol!(Erc20, "src/abis/Erc20Abi.json");

struct Transfer {
    address: Address,
    from: Address,
    to: Address,
    value: U256,
}

pub struct TransfersHandler {
    topic: B256,
}

impl TransfersHandler {
    fn get_topic() -> B256 {
        let topic = env::var("TRANSFER_TOPIC").unwrap();
        return B256::from_str(&topic).unwrap();
    }

    pub fn new() -> Self {
        return Self {
            topic: Self::get_topic(),
        };
    }

    fn equals_topic(&self, topic: B256) -> bool {
        return self.topic == topic;
    }

    fn get_transfer(&self, log_data: &PrimitivesLog) -> Option<Transfer> {
        let transfer = Erc20::Transfer::decode_log(log_data, false);

        let transfer = match transfer {
            Ok(transfer) => transfer,
            _ => return None,
        };

        return Some(Transfer {
            address: transfer.address,
            from: transfer.from,
            to: transfer.to,
            value: transfer.value,
        });
    }

    fn display_transfer(&self, transfer: Transfer) {
        println!();
        println!("New Transfer Handled");
        println!("Address: {}", transfer.address);
        println!("From: {}", transfer.from);
        println!("To: {}", transfer.to);
        println!("Value: {}", transfer.value);
        println!();
    }

    pub fn handle(&self, logs: &Vec<TypesLog>) {
        for log in logs {
            let topic = match log.topic0() {
                Some(topic) => *topic,
                _ => continue,
            };

            let equals_topic = self.equals_topic(topic);

            match equals_topic {
                false => continue,
                _ => {}
            };

            let transfer = self.get_transfer(&log.inner);

            let transfer = match transfer {
                Some(transfer) => transfer,
                _ => continue,
            };

            self.display_transfer(transfer);
        }
    }
}
