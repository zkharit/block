use std::collections::HashMap;
use std::env;
use std::time::SystemTime;
use std::path::PathBuf;

use k256::PublicKey;

use crate::account::Account;
use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::config::{Config, NetworkConfig, ValidatorConfig, WalletConfig};
use crate::network::Network;
use crate::transaction::Transaction;
use crate::util::read_string;
use crate::validator_account::ValidatorAccount;
use crate::validator::Validator;
use crate::verification_engine;
use crate::wallet::Wallet;

use crate::constants::{BLOCK_ADDRESS_SIZE, BLOCK_VERSION, COMPRESSED_PUBLIC_KEY_SIZE, DEFAULT_CONFIG_FILE_NAME, LOOSE_CHANGE_RECIPIENT, LOWEST_DENOMINATION_PER_COIN, MAX_TRANSACTIONS_PER_BLOCK, VALIDATOR_ENABLE_RECIPIENT};

pub struct Controller {
    config: Config,
    blockchain: Blockchain,
    wallet:  Wallet,
    validator: Validator,
    network: Network,
    // storage: Storage,
}

impl Controller {
    pub fn new() -> Option<Self> {
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

        // initialize wallet
        println!("Initializing wallet");
        let mut wallet: Wallet = Wallet::new(config.get_wallet_config());
        println!("Initialized wallet");
        println!("Wallet address: {}", wallet.get_address_string());
        println!();

        // initialize validator
        println!("Initializing validator");
        let mut validator = Validator::new(config.get_validator_config());
        println!("Initialized validator");
        println!();

        // initialize network
        println!("Initializing network");
        let mut network = Network::new(config.get_network_config());
        println!("Initialized network");
        println!();

        if !network.get_local_blockchain() {
            // test connection to peers, remove them from peer list if unable to connect
            println!("Testing connection to peers");
            network.initial_connect();
            println!("Finished testing connection to peers");
            println!();
        }

        println!("Initializing blockchain");
        let mut blockchain = Blockchain::new();
        println!("Initialized blockchain");
        println!();

        // if the config specifies to generate a local blockchain, or unable to connect to any peers
        if network.get_local_blockchain() || network.get_peer_list_len() == 0 {
            println!("YOU SHOULD CONFIRM YOUR WALLET NONCE IS SET TO 0 BEFORE CREATING A LOCAL BLOCKCHAIN");
            if !network.get_local_blockchain() {
                loop {
                    // prompt user if they want to create their own local blockchain because they were unable to connect to any peers
                    println!("Unable to connect to peers listed in config file, would you like to create a local blockchain instead? (yes/no)");
                    let blockchain_input = read_string().to_lowercase();
                    println!();

                    match blockchain_input.as_str() {
                        "yes" => break,
                        "no" => {
                            println!("Please check your network and/or update your peer_list in the network section of your config file");
                            println!();
                            return None
                        },
                        _ => continue
                    }
                }
            }

            // create a local blockchain with the generated wallet as the initial validator
            println!("Creating local blockchain, using generated wallet as initial validator in genesis block");
            println!();

            // create initial coinbase transaction
            let genesis_coinbase_tx = wallet.create_coinbase_tx(verification_engine::get_block_subsidy(0), wallet.get_address()).unwrap();
            // create initial validator enable transaction
            let genesis_validator_enable_tx = wallet.create_validator_enable_tx(0, 0).unwrap();
            // add the initial validator enable transaction to the genesis block transaction vector
            let mut genesis_tx_vec = vec![genesis_coinbase_tx, genesis_validator_enable_tx];
            // get the current timestamp
            let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(timestamp) => timestamp.as_secs(),
                Err(_) => return None,
            };
            // get the local blockchain genesis block signature
            let genesis_sig = match wallet.create_block_sig(*BLOCK_VERSION, [0x00; 32], timestamp, &genesis_tx_vec) {
                Some(genesis_sig) => genesis_sig,
                None => {
                    println!("Unable to sign local blockchain genesis block, check your wallet file");
                    println!();
                    return None
                }
            };
            // create the local blockchain genesis block
            let genesis_block = validator.create_block(&mut genesis_tx_vec, [0x00; 32], timestamp, genesis_sig);

            // increment the wallet nonce to account for the validator enable transaction
            wallet.increment_nonce();

            // add local genesis block to blockchain
            println!("Adding local blockchain genesis block to blockchain");
            blockchain.add_local_genesis_block(&genesis_block);
            println!("Added local blockchain genesis block to blockchain");
            println!();
        } else {
            // initialize blockchain with typical genesis block
            println!("Adding genesis block to blockchain");
            blockchain.add_genesis_block();
            println!("Added genesis block to blockchain");
            println!();

            // restore blockchain from network
            println!("Restoring Blockchain");
            // ToDo:
            // println!("Restored Blockchain: {:X?}", blockchain);
            println!();
        }

        Some(
            Self {
                config,
                blockchain,
                wallet,
                validator,
                network
            })
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
        println!("{:X?}", self.blockchain);
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

    pub fn blockchain_get_mempool(&self) -> HashMap<[u8; COMPRESSED_PUBLIC_KEY_SIZE], Vec<Transaction>> {
        self.blockchain.get_mempool_clone()
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

    pub fn blockchain_prune_mempool(&mut self) {
        self.blockchain.clear_mempool()
    }

    pub fn transaction_create_a_b(&mut self, recipient: [u8; BLOCK_ADDRESS_SIZE], amount: u64, fee: u64) -> bool {
        // ToDo: need to broadcast transaction to network
        let tx = match self.wallet.create_tx(amount, fee, recipient) {
            Some(tx) => tx,
            None => return false
        };

        // add the transaction to the mempool
        if !self.blockchain.add_transaction_mempool(&tx) {
            return false
        }

        // broadcast the transaction to the network
        if !self.network.broadcast_transaction(&tx) {
            return false
        }

        // increment the wallet nonce
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

    pub fn about_network_config(&self) -> NetworkConfig {
        self.config.get_network_config()
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
        match self.wallet.create_coinbase_tx(verification_engine::get_block_subsidy(block_height + 1), self.wallet.get_address()) {
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

        // ToDo: need a way to prune uneeded accounts (can only be done when storage is implemented because of nonce), unhelpful validators from blockchain

        // get a list of the accounts on the blockchain, used for nonce checking later
        let mut blockchain_accounts = self.blockchain.get_accounts();

        // get a reference to the mempool
        let mempool = self.blockchain.get_mempool();

        // iterate through the mempool while there are transactions left in it, or the tx_vec is at MAX_TRANSACTION-PER_BLOCK
        while !mempool.is_empty() && tx_vec.len() < *MAX_TRANSACTIONS_PER_BLOCK {
            // hold the address and max fee found from the first transaction of each account's transaction vector in the mempool hashmap
            let mut max_transaction_fee_sender: [u8; COMPRESSED_PUBLIC_KEY_SIZE] = [0x00; COMPRESSED_PUBLIC_KEY_SIZE];
            let mut max_transaction_fee: Option<u64> = None;

            // iterate through each account that has a transaction in the mempool, and check its first (earliest nonce) transaction and check if its fee is higher than the max already found fee
            for (sender, transactions) in mempool.clone() {
                match max_transaction_fee {
                    // if there already is a max_transaction_fee compare the current transaction's fee to the max transaction fee
                    Some(max_fee) => {
                        // if the current transaction's fee is higher than the max trnasaction fee then confirm the nonce is correct for this transaction
                        if transactions[0].fee > max_fee {
                            // get the account public key
                            let account_pub_key = match PublicKey::from_sec1_bytes(&sender) {
                                Ok(account_pub_key) => account_pub_key,
                                // if an invalid public key is received then the transaction is invalid
                                Err(_) => continue
                            };

                            // get the address for the account public key
                            let account_address = Wallet::generate_address(&account_pub_key, true);

                            // obtain the account nonce in the blockchains view
                            let tx_account_nonce = match blockchain_accounts.get(&account_address) {
                                Some(tx_account) => {
                                    tx_account.get_nonce()
                                }
                                None => {
                                    // if the account isn't in the blockchain yet, then the nonce needs to be 0
                                    0
                                }
                            };

                            if tx_account_nonce == transactions[0].nonce {
                                // if the transaction nonce matches the account nonce on the blockchain then mark this transaction as the new max fee
                                max_transaction_fee_sender = sender;
                                max_transaction_fee = Some(transactions[0].fee);
                            } 

                            if transactions[0].nonce < tx_account_nonce {
                                // if the user has sent a transaction with a nonce less than their account nonce, remove it from the mempool because this will never be a valid transaction
                                // get the sender's tx vec
                                let sender_tx_vec = mempool.get_mut(&sender).unwrap();

                                // remove the transaction from the mempool
                                sender_tx_vec.remove(0);

                                // if the sender has no more transactions remove their entry from the mempool hash map
                                if sender_tx_vec.len() == 0 {
                                    mempool.remove(&sender);
                                }
                            }
                        }
                    },
                    // if there is no max transaction fee yet then add the first transaction that has a valid nonce
                    None => {
                        // get the account public key
                        let account_pub_key = match PublicKey::from_sec1_bytes(&sender) {
                            Ok(account_pub_key) => account_pub_key,
                            // if an invalid public key is received then the transaction is invalid
                            Err(_) => continue
                        };

                        // get the address for the account public key
                        let account_address = Wallet::generate_address(&account_pub_key, true);

                        // obtain the account nonce in the blockchains view
                        let tx_account_nonce = match blockchain_accounts.get(&account_address) {
                            Some(tx_account) => {
                                tx_account.get_nonce()
                            }
                            None => {
                                // if the account isn't in the blockchain yet, then the nonce needs to be 0
                                0
                            }
                        };

                        // if the transaction nonce matches the account nonce on the blockchain then mark this transaction as the new max fee
                        if tx_account_nonce == transactions[0].nonce {
                            max_transaction_fee_sender = sender;
                            max_transaction_fee = Some(transactions[0].fee);

                            continue;
                        }
                        
                        if transactions[0].nonce < tx_account_nonce {
                            // if the user has sent a transaction with a nonce less than their account nonce, remove it from the mempool because this will never be a valid transaction
                            // get the sender's tx vec
                            let sender_tx_vec = mempool.get_mut(&transactions[0].sender).unwrap();

                            // remove the transaction from the mempool
                            sender_tx_vec.remove(0);

                            // if the sender has no more transactions remove their entry from the mempool hash map
                            if sender_tx_vec.len() == 0 {
                                mempool.remove(&transactions[0].sender);
                            }
                        }
                    }
                }
            }

            // if the max fee transaction was updated then add it to the tx vec and remove it from the mempool
            if max_transaction_fee_sender != [0x00; COMPRESSED_PUBLIC_KEY_SIZE] {
                // get the sender's tx vec with the highest transaction fee for this iteration
                let sender_tx_vec = mempool.get_mut(&max_transaction_fee_sender).unwrap();

                // push their transaction into the tx_vec for the block
                tx_vec.push(sender_tx_vec[0].clone());

                // remove the transaction from the mempool
                sender_tx_vec.remove(0);

                // if the sender has no more transactions remove their entry from the mempool hash map
                if sender_tx_vec.len() == 0 {
                    mempool.remove(&max_transaction_fee_sender);
                }

                // after a transaction is added to the block update the account nonce (in the blockchain account CLONE NOT the actual blockchain) so that multiple transactions per account can be added per block
                // get the account public key
                let account_pub_key = match PublicKey::from_sec1_bytes(&max_transaction_fee_sender) {
                    Ok(account_pub_key) => account_pub_key,
                    // if an invalid public key is received then the transaction is invalid
                    Err(_) => continue
                };

                // get the address for the account public key
                let account_address = Wallet::generate_address(&account_pub_key, true);

                blockchain_accounts.get_mut(&account_address).unwrap().increase_nonce();
            } else {
                // if the max fee transaction was never updated then there are no valid transactions in the mempool
                break;
            }
        }

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