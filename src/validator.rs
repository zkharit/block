use crate::config::ValidatorConfig;
use crate::transaction::Transaction;
use crate::verification_engine;
use crate::wallet::Wallet;

pub struct Validator {
    // config for the validator
    config: ValidatorConfig,
    // wallet to be used for validation purposes
    wallet: Wallet
}

impl Validator {
    // ToDo: need to look into not cloneing the wallet here, this could result in inconsistent nonce's
    pub fn new(config: ValidatorConfig, wallet: &Wallet) -> Self {
        Self {
            config,
            wallet: wallet.clone(),
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

    pub fn create_block(&mut self, transactions: Vec<Transaction>, prev_hash: [u8; 32], block_height: u64) {
        // ToDo:
        let coinbase_tx = self.create_coinbase_tx(block_height);
    }
}

