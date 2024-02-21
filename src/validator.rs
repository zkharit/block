use std::time::SystemTime;

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::config::ValidatorConfig;
use crate::transaction::Transaction;
use crate::verification_engine;
use crate::wallet::Wallet;

use crate::constants::{MAX_TRANSACTIONS_PER_BLOCK, BLOCK_VERSION};

pub struct Validator<'a> {
    // config for the validator
    config: ValidatorConfig,
    // wallet to be used for validation purposes
    wallet: &'a mut Wallet,
    // reference to the current blockchain
    blockchain: &'a mut Blockchain
}

impl <'a> Validator <'a> {
    pub fn new(config: ValidatorConfig, wallet: &'a mut Wallet, blockchain: &'a mut Blockchain) -> Self {
        Self {
            config,
            wallet,
            blockchain,
        }
    }

    pub fn create_coinbase_tx(&mut self, block_height: u64) -> Option<Transaction> {
        let coinbase_tx = match self.wallet.create_coinbase_tx(verification_engine::get_block_subsidy(block_height), self.wallet.get_address()) {
            Some(coinbase_tx) => coinbase_tx,
            None => {
                println!("Failed to create coinbase tx");
                return None
            }
        };

        Some(coinbase_tx)
    }

    pub fn create_block(&mut self) -> Option<Block> {
        // create tx vector
        let mut tx_vec: Vec<Transaction> = vec![];

        // create coinbase transaction for this block
        let coinbase_tx = match self.create_coinbase_tx(self.blockchain.get_block_height()) {
            Some(coinbase_tx) => tx_vec.push(coinbase_tx),
            // if coinbase_tx cannot be created either do not propose the block or propose it without a coinbase transaction depending on config
            None =>  {
                if self.config.get_propose_without_coinbase() {
                    ()
                } else {
                    return None
                }
            }
        };

        // get a clone the mempool
        let mut mempool_transactions = self.blockchain.get_mempool();

        // truncate the mempool to the first MAX_TRANSACTIONS_PER_BLOCK - 1 (because of the coinbase) to the block
        mempool_transactions.truncate(MAX_TRANSACTIONS_PER_BLOCK - 1);

        // remove as many transactions from the mempool that are to be added to the upcoming block
        self.blockchain.remove_from_mempool(mempool_transactions.len() as u64);

        // add the truncated ordered mempool to the transaction vector
        tx_vec.append(&mut mempool_transactions);

        // get the current timestamp
        let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(timestamp) => timestamp.as_secs(),
            Err(_) => return None,
        };

        // get the hash of the previous block
        let prev_hash = self.blockchain.get_last_block().serialize_hash_block_header().try_into().unwrap();

        // create the block signature
        let block_sig = match self.wallet.create_block_sig(*BLOCK_VERSION, prev_hash, timestamp, &tx_vec) {
            Some(block_sig) => block_sig,
            None => return None
        };

        // create the new block
        let block = Block::new(*BLOCK_VERSION, prev_hash, timestamp, &tx_vec, block_sig);
        
        Some(block)
    }
}

