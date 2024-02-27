use std::time::SystemTime;
use std::{path::PathBuf, env};

use crate::account::Account;
use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::config::{Config, ValidatorConfig, WalletConfig};
use crate::transaction::Transaction;
use crate::validator_account::ValidatorAccount;
use crate::validator::Validator;
use crate::verification_engine;
use crate::wallet::Wallet;

use crate::constants::{BLOCK_ADDRESS_SIZE, BLOCK_VERSION, DEFAULT_CONFIG_FILE_NAME, LOOSE_CHANGE_RECIPIENT, LOWEST_DENOMINATION_PER_COIN, MAX_TRANSACTIONS_PER_BLOCK, VALIDATOR_ENABLE_RECIPIENT};

pub struct Controller {
    config: Config,
    blockchain: Blockchain,
    wallet:  Wallet,
    validator: Validator,
    // network: Network,
    // storage: Storage,
}

impl Controller {
    pub fn new() -> Self {
        // get config information
        let args: Vec<String> = env::args().collect();
        let config: Config;
        if args.len() == 2 { 
            config = Config::new(&PathBuf::from(&args[1]));
            println!("Using config file: {:?}", &PathBuf::from(&args[1]));
        } else {
            config = Config::new(&PathBuf::from(DEFAULT_CONFIG_FILE_NAME));
            println!("Using config file: {:?}", &PathBuf::from(DEFAULT_CONFIG_FILE_NAME));
        }
        println!();

        // initialize blockchain
        println!("Initializing blockchain with genesis block");
        let blockchain = Blockchain::new();
        println!("Initialized blockchain:");
        println!("{:X?}", blockchain);
        println!();

        // initialize wallet
        println!("Initializing wallet");
        let wallet: Wallet = Wallet::new(config.get_wallet_config());
        println!("Initialized wallet");
        println!("Wallet address: {}", wallet.get_address_string());
        println!();

        // initialize validator
        println!("Initializing validator");
        let validator = Validator::new(config.get_validator_config());
        println!("Initialized validator");
        println!();

        // initialize network
        println!("Initializing network");
        // ToDo:
        println!("Initialized network");
        println!();

        // restore blockchain from network
        println!("Restoring Blockchain");
        // ToDo:
        // println!("Restored Blockchain: {:X?}", blockchain);
        println!();

        Self {
            config,
            blockchain,
            wallet,
            validator
        }
    }

    pub fn wallet_overview(&self) {
        println!("Wallet:");
        println!("\tAddress: {}", self.wallet.get_address_string());
        println!("\tBalance: {:.8}", self.wallet_get_balance());
        println!("\tNonce: {}", self.wallet_get_nonce());
    }

    pub fn wallet_get_address(&self) -> [u8;39] {
        self.wallet.get_address()
    }

    pub fn wallet_get_address_string(&self) -> String {
        self.wallet.get_address_string()
    }

    pub fn wallet_get_balance(&self) -> f64 {
        match self.blockchain.get_account(&self.wallet_get_address()) {
            Some(account) => account.get_balance() as f64 / LOWEST_DENOMINATION_PER_COIN,
            None => 0.0
        }
    }

    pub fn wallet_get_nonce(&self) -> u64 {
        self.wallet.get_nonce()
    }

    pub fn wallet_get_private_key(&self) -> Option<String> {
        match self.wallet.get_private_key_string() {
            Some(private_key_string) => Some(private_key_string),
            None => None
        }
    }   

    pub fn wallet_set_nonce(&mut self, nonce: u64) {
        self.wallet.set_nonce(nonce);
    }

    pub fn blockchain_overview(&self) {
        println!("Blockchain address: {:X?}", self.blockchain);
    }

    pub fn blockchain_get_block_height(&self) -> u64 {
        self.blockchain.get_block_height()
    }

    pub fn blockchain_get_block(&self, block_height: u64) -> Option<Block> {
        self.blockchain.get_block(block_height)
    }

    pub fn blockchain_get_account(&self, address: &[u8; BLOCK_ADDRESS_SIZE]) -> Option<&Account> {
        self.blockchain.get_account(address)
    }

    pub fn blockchain_get_mempool(&self) -> Vec<Transaction> {
        self.blockchain.get_mempool()
    }

    pub fn blockchain_get_validators(&self) -> Vec<ValidatorAccount> {
        self.blockchain.get_validators()
    }

    pub fn blockchain_get_total_staked(&self) -> u64 {
        match self.blockchain.get_account(&VALIDATOR_ENABLE_RECIPIENT) {
            Some(account) => account.get_balance(),
            None => 0
        }
    }

    pub fn blockchain_get_total_change(&self) -> u64 {
        match self.blockchain.get_account(&LOOSE_CHANGE_RECIPIENT) {
            Some(account) => account.get_balance(),
            None => 0
        }
    }

    pub fn transaction_create_a_b(&mut self, recipient: [u8; BLOCK_ADDRESS_SIZE], amount: u64, fee: u64) -> bool {
        // ToDo: need to broadcast transaction to network
        let tx = match self.wallet.create_tx(amount, fee, recipient) {
            Some(tx) => tx,
            None => return false
        };

        if !self.blockchain.add_transaction_mempool(&tx) {
            return false
        }

        self.wallet.increment_nonce();
        true
    }

    pub fn transaction_create_validator_enable(&mut self, amount: u64, fee: u64) -> bool {
        let tx = match self.wallet.create_validator_enable_tx(amount, fee) {
            Some(tx) => tx,
            None => return false
        };


        if !self.blockchain.add_transaction_mempool(&tx) {
            return false
        }

        self.wallet.increment_nonce();
        true
    }

    pub fn transaction_create_validator_revoke(&mut self, amount: u64, fee: u64) -> bool {
        let tx = match self.wallet.create_validator_revoke_tx(amount, fee) {
            Some(tx) => tx,
            None => return false
        };


        if !self.blockchain.add_transaction_mempool(&tx) {
            return false
        }

        self.wallet.increment_nonce();
        true
    }

    pub fn about_wallet_config(&self) -> WalletConfig {
        self.config.get_wallet_config()
    }

    pub fn about_validator_config(&self) -> ValidatorConfig {
        self.config.get_validator_config()
    }

    pub fn check_address_checksum(&self, address: [u8; BLOCK_ADDRESS_SIZE]) -> bool {
        self.wallet.check_address_checksum(address)
    }

    pub fn validator_create_block(&mut self) -> Option<Block> {
        // get the current block height
        let block_height = self.blockchain.get_block_height();

        let prev_hash = self.blockchain.get_last_block().serialize_hash_block_header().try_into().unwrap();

        let mut tx_vec: Vec<Transaction> = vec![];

        // create coinbase transaction for this block
        match self.wallet.create_coinbase_tx(verification_engine::get_block_subsidy(block_height), self.wallet.get_address()) {
            Some(coinbase_tx) => tx_vec.push(coinbase_tx),
            // if coinbase_tx cannot be created either do not propose the block or propose it without a coinbase transaction depending on config
            None =>  {
                if self.validator.get_config().get_propose_without_coinbase() {
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

        // create the block signature
        let block_sig = match self.wallet.create_block_sig(*BLOCK_VERSION, prev_hash, timestamp, &tx_vec) {
            Some(block_sig) => block_sig,
            None => return None
        };
        
        Some(self.validator.create_block(&mut tx_vec, prev_hash, timestamp, block_sig))
    }
}