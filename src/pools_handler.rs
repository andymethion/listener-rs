use std::{env, str::FromStr};

use alloy::{
    primitives::{Address, Log as PrimitivesLog, B256},
    rpc::types::Log as TypesLog,
    sol,
    sol_types::SolEvent,
};

sol!(UniswapV2Factory, "src/abis/UniswapV2FactoryAbi.json");
sol!(UniswapV3Factory, "src/abis/UniswapV3FactoryAbi.json");

struct Pool {
    address: Address,
    token0: Address,
    token1: Address,
}

pub struct PoolsHandler {
    topic2: B256,
    topic3: B256,
}

impl PoolsHandler {
    fn get_topic2() -> B256 {
        let topic2 = env::var("PAIR2_TOPIC").unwrap();
        return B256::from_str(&topic2).unwrap();
    }

    fn get_topic3() -> B256 {
        let topic3 = env::var("POOL3_TOPIC").unwrap();
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

    fn display_pool2(&self, pool2: Pool) {
        println!();
        println!("New Pool2 Handled");
        println!("Address: {}", pool2.address);
        println!("Token0: {}", pool2.token0);
        println!("Token1: {}", pool2.token1);
        println!();
    }

    fn display_pool3(&self, pool3: Pool) {
        println!();
        println!("New Pool3 Handled");
        println!("Address: {}", pool3.address);
        println!("Token0: {}", pool3.token0);
        println!("Token1: {}", pool3.token1);
        println!();
    }

    fn get_pool2(&self, log_data2: &PrimitivesLog) -> Pool {
        let pool2 = UniswapV2Factory::PairCreated::decode_log(log_data2, false).unwrap();
        return Pool {
            address: pool2.pair,
            token0: pool2.token0,
            token1: pool2.token1,
        };
    }

    fn get_pool3(&self, log_data3: &PrimitivesLog) -> Pool {
        let pool3 = UniswapV3Factory::PoolCreated::decode_log(log_data3, false).unwrap();
        return Pool {
            address: pool3.pool,
            token0: pool3.token0,
            token1: pool3.token1,
        };
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

        let pool2 = self.get_pool2(&log2.inner);

        self.display_pool2(pool2);
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

        let pool3 = self.get_pool3(&log3.inner);

        self.display_pool3(pool3);
    }

    pub fn handle(&self, logs: &Vec<TypesLog>) {
        for log in logs {
            self.handle2(log);
            self.handle3(log);
        }
    }
}
