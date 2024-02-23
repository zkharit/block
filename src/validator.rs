use k256::ecdsa::Signature;

use crate::block::Block;
use crate::config::ValidatorConfig;
use crate::transaction::Transaction;

use crate::constants::BLOCK_VERSION;

pub struct Validator {
    // config for the validator
    config: ValidatorConfig,
}

impl Validator {
    pub fn new(config: ValidatorConfig) -> Self {
        Self {
            config,
        }
    }

    pub fn get_config(&self) -> ValidatorConfig {
        self.config.clone()
    }

    pub fn create_block(&mut self, transactions: &mut Vec<Transaction>, prev_hash: [u8; 32], timestamp: u64, block_sig: Signature) -> Block {
        // create the new block
        Block::new(*BLOCK_VERSION, prev_hash, timestamp, &transactions, block_sig)
    }
}

