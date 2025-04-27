use std::{env, str::FromStr};

use alloy::{
    primitives::{Address, Log as PrimitivesLog, B256, I256},
    rpc::types::Log as TypesLog,
    sol,
    sol_types::SolEvent,
};

sol!(UniswapV2PairAbi, "src/abis/UniswapV2PairAbi.json");
sol!(UniswapV3PoolAbi, "src/abis/UniswapV3PoolAbi.json");

struct Swap {
    address: Address,
    amount0: I256,
    amount1: I256,
}

pub struct SwapsHandler {
    topic2: B256,
    topic3: B256,
}

impl SwapsHandler {
    fn get_topic2() -> B256 {
        let topic2 = env::var("SWAP2_TOPIC").unwrap();
        return B256::from_str(&topic2).unwrap();
    }

    fn get_topic3() -> B256 {
        let topic3 = env::var("SWAP3_TOPIC").unwrap();
        return B256::from_str(&topic3).unwrap();
    }

    pub fn new() -> Self {
        return Self {
            topic2: Self::get_topic2(),
            topic3: Self::get_topic3(),
        };
    }

    fn equals_topic2(&self, topic2: B256) -> bool {
        return self.topic2 == topic2;
    }

    fn equals_topic3(&self, topic3: B256) -> bool {
        return self.topic3 == topic3;
    }

    fn get_amounts2(&self, swap2: &PrimitivesLog<UniswapV2PairAbi::Swap>) -> (I256, I256) {
        match swap2.amount0In.is_zero() {
            true => {
                return (
                    I256::from_str(&swap2.amount0Out.to_string()).unwrap() * I256::MINUS_ONE,
                    I256::from_str(&swap2.amount1In.to_string()).unwrap(),
                );
            }
            false => {
                return (
                    I256::from_str(&swap2.amount0In.to_string()).unwrap(),
                    I256::from_str(&swap2.amount1Out.to_string()).unwrap() * I256::MINUS_ONE,
                );
            }
        };
    }

    fn get_swap2(&self, log_data2: &PrimitivesLog) -> Swap {
        let swap2 = UniswapV2PairAbi::Swap::decode_log(log_data2, false).unwrap();

        let amounts2 = self.get_amounts2(&swap2);

        return Swap {
            address: swap2.address,
            amount0: amounts2.0,
            amount1: amounts2.1,
        };
    }

    fn get_swap3(&self, log_data3: &PrimitivesLog) -> Swap {
        let swap3 = UniswapV3PoolAbi::Swap::decode_log(log_data3, false).unwrap();
        return Swap {
            address: swap3.address,
            amount0: swap3.amount0,
            amount1: swap3.amount1,
        };
    }

    fn display_swap2(&self, swap2: Swap) {
        println!();
        println!("New Swap2 Handled");
        println!("Address: {}", swap2.address);
        println!("Amount0: {}", swap2.amount0);
        println!("Amount1: {}", swap2.amount1);
        println!();
    }

    fn display_swap3(&self, swap3: Swap) {
        println!();
        println!("New Swap3 Handled");
        println!("Address: {}", swap3.address);
        println!("Amount0: {}", swap3.amount0);
        println!("Amount1: {}", swap3.amount1);
        println!();
    }

    fn handle2(&self, log2: &TypesLog) {
        let topic2 = match log2.topic0() {
            Some(topic2) => *topic2,
            _ => return,
        };

        let equals_topic2 = self.equals_topic2(topic2);

        match equals_topic2 {
            false => return,
            _ => {}
        };

        let swap2 = self.get_swap2(&log2.inner);

        self.display_swap2(swap2);
    }

    fn handle3(&self, log3: &TypesLog) {
        let topic3 = match log3.topic0() {
            Some(topic3) => *topic3,
            _ => return,
        };

        let equals_topic3 = self.equals_topic3(topic3);

        match equals_topic3 {
            false => return,
            _ => {}
        };

        let swap3 = self.get_swap3(&log3.inner);

        self.display_swap3(swap3);
    }

    pub fn handle(&self, logs: &Vec<TypesLog>) {
        for log in logs {
            self.handle2(log);
            self.handle3(log);
        }
    }
}
