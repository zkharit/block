use std::collections::HashMap;

use k256::PublicKey;

use crate::account::Account;
use crate::block::Block;
use crate::transaction::Transaction;
use crate::verification_engine;
use crate::wallet::Wallet;

use crate::constants::{BLOCK_ADDRESS_SIZE, GENESIS_BLOCK, LOOSE_CHANGE_RECEVIER, VALIDATOR_ENABLE_RECIPIENT};

#[derive(Debug, Clone)]
pub struct Blockchain {
    // ToDo: limit total blocks stored in memory
    // vector of most recent x blocks
    blocks: Vec<Block>,
    // hashmap of all accounts on the blockchain
    accounts: HashMap<[u8; BLOCK_ADDRESS_SIZE], Account>,
    // the current blockheight
    block_height: u64
}

impl Blockchain {
    pub fn new() -> Self {
        // create genesis block
        let mut blocks: Vec<Block> = vec![];
        let genesis_block = Block::from(GENESIS_BLOCK.to_vec()).unwrap();

        // add genesis block to chain
        blocks.push(genesis_block.clone());

        // create account set
        let accounts = HashMap::new();

        let block_height = 0;

        // create blockchain object
        let mut blockchain = Blockchain {
            blocks,
            accounts,
            block_height
        };

        // update blockchain object with genesis block transaction
        blockchain.update_chain(&genesis_block);

        blockchain
    }

    pub fn add_block(self, block: &Block) -> (bool, Blockchain) {
        // temporary blockchain to make changes on, will return this blockchain if block and all transactions are valid
        let mut new_blockchain = self.clone();

        // run the block through the verification engine to ensure that it is a valid block
        if !verification_engine::verify_block(block.clone(), &new_blockchain) {
            return (false, self)
        }

        // add block to blockchain
        new_blockchain.blocks.push(block.clone());

        // update chain state
        if !new_blockchain.update_chain(block) {
            return (false, self)
        }

        // increment the block height
        new_blockchain.increase_block_height();

        (true, new_blockchain)
    }

    fn update_chain(&mut self, block: &Block) -> bool {
        // this function assumes that the blocks given to it are valid with the current chain state
        // this function should only ever be called after the verification engine has verified all transactions within the given block with the current chain state

        // iterate through all transactions of the given block
        for transaction in block.get_transactions() {
            if !self.update_chain_transaction(transaction, block) {
                return false
            }
        }

        true
    }

    pub fn update_chain_transaction(&mut self, transaction: &Transaction, block: &Block) -> bool {
        // get the validator address for this block
        let validator_address: [u8; BLOCK_ADDRESS_SIZE] = match block.get_transactions().get(0) {
            Some(transaction) => {
                if verification_engine::is_coinbase(transaction, block, self.get_block_height()) {
                    transaction.recipient
                } else {
                    // if there is no coinbase transaction then the validator will lose all rewards for this block
                    *LOOSE_CHANGE_RECEVIER
                }
            },
            // if there are no transactions in the block then the validator will lose all rewards for this block
            None => *LOOSE_CHANGE_RECEVIER
        };

        // transaction is a validator enable transaction
        if verification_engine::is_validator_enable(&transaction, self) {
            // increase the block validator's balance by the transaction fee
            match self.accounts.get_mut(&validator_address) {
                // increase the block validator's balance by the transaction fee
                Some(account) => account.increase_balance(transaction.fee),
                None => {
                    // create a new account for this newly discovered address
                    self.accounts.insert(validator_address, Account::new(validator_address));
                    // increase the block validator's balance by the transaction fee
                    self.accounts.get_mut(&validator_address).unwrap().increase_balance(transaction.fee);
                }
            };

            // get the account public key
            let account_pub_key = match PublicKey::from_sec1_bytes(&transaction.sender) {
                Ok(account_pub_key) => account_pub_key,
                // This should NEVER happen since this block must have been validated by the verification_engine first
                Err(_) => return false
            };

            // get the address for the account public key
            let account_address = Wallet::generate_address(&account_pub_key, true);

            // increment the account nonce, set the account as a validator, and set the stake as the transaction amount
            match self.accounts.get_mut(&account_address) {
                Some(account) => {
                    account.increase_nonce();
                    account.decrease_balance(transaction.amount + transaction.fee);
                    account.set_stake(transaction.amount);
                    account.set_validator(true);
                },
                None => {
                    // create new account for this newly discovered address
                    self.accounts.insert(account_address, Account::new(account_address));

                    let account = self.accounts.get_mut(&account_address).unwrap();
                    account.increase_nonce();
                    account.decrease_balance(transaction.amount + transaction.fee);
                    account.set_stake(transaction.amount);
                    account.set_validator(true);
                }
            };

            // increase the balance of the VALIDATOR_ENABLE_RECIPIENT
            match self.accounts.get_mut(VALIDATOR_ENABLE_RECIPIENT) {
                Some(account) => {
                    // increase the VALIDATOR_ENABLE_RECIPIENT's balance
                    account.increase_balance(transaction.amount);
                },
                None => {
                    // create new account for this newly discovered address
                    self.accounts.insert(*VALIDATOR_ENABLE_RECIPIENT, Account::new(*VALIDATOR_ENABLE_RECIPIENT));

                    // increase the VALIDATOR_ENABLE_RECIPIENT's balance
                    let account = self.accounts.get_mut(VALIDATOR_ENABLE_RECIPIENT).unwrap();
                    account.increase_balance(transaction.amount);
                }
            };
        }
        // transaction is a validator revoke trasnaction
        else if verification_engine::is_validator_revoke(&transaction, self) {
            // increase the block validator's balance by the transaction fee
            match self.accounts.get_mut(&validator_address) {
                // increase the block validator's balance by the transaction fee
                Some(account) => account.increase_balance(transaction.fee),
                None => {
                    // create a new account for this newly discovered address
                    self.accounts.insert(validator_address, Account::new(validator_address));
                    // increase the block validator's balance by the transaction fee
                    self.accounts.get_mut(&validator_address).unwrap().increase_balance(transaction.fee);
                }
            };

            // get the account public key
            let account_pub_key = match PublicKey::from_sec1_bytes(&transaction.sender) {
                Ok(account_pub_key) => account_pub_key,
                // This should NEVER happen since this block must have been validated by the verification_engine first
                Err(_) => return false
            };

            // get the address for the account public key
            let account_address = Wallet::generate_address(&account_pub_key, true);

            // increment the nonce, set the account as not a validator, increase their balance by the transaction amount, and set their current stake back to 0
            match self.accounts.get_mut(&account_address) {
                Some(account) => {
                    account.increase_nonce();
                    account.set_stake(0);
                    account.decrease_balance(transaction.fee);
                    account.increase_balance(transaction.amount);
                    account.set_validator(false);
                },
                None => {
                    // create a new account for this newly discovered address
                    self.accounts.insert(account_address, Account::new(account_address));

                    let account = self.accounts.get_mut(&account_address).unwrap();
                    account.increase_nonce();
                    account.set_stake(0);
                    account.decrease_balance(transaction.fee);
                    account.increase_balance(transaction.amount);
                    account.set_validator(false);
                }
            };

            // decrease the balance of the VALIDATOR_ENABLE_RECIPIENT
            match self.accounts.get_mut(VALIDATOR_ENABLE_RECIPIENT) {
                Some(account) => {
                    // decrease the VALIDATOR_ENABLE_RECIPIENT's balance
                    account.decrease_balance(transaction.amount);
                },
                None => {
                    // create new account for this newly discovered address
                    self.accounts.insert(*VALIDATOR_ENABLE_RECIPIENT, Account::new(*VALIDATOR_ENABLE_RECIPIENT));

                    // decrease the VALIDATOR_ENABLE_RECIPIENT's balance
                    let account = self.accounts.get_mut(VALIDATOR_ENABLE_RECIPIENT).unwrap();
                    account.decrease_balance(transaction.amount);
                }
            };
        } else {
            // transaction is a coinbase transaction
            if verification_engine::is_coinbase(&transaction, block, self.get_block_height()) {
                match self.accounts.get_mut(&transaction.recipient) {
                    // increase the balance by the coinbase amount
                    Some(account) => account.increase_balance(transaction.amount),
                    None => {
                        // create a new account for this newly discovered address
                        self.accounts.insert(transaction.recipient, Account::new(transaction.recipient));
                        // update the balance of the recipient account
                        self.accounts.get_mut(&transaction.recipient).unwrap().increase_balance(transaction.amount);
                    }
                };
            }
            // transaction is a typical A -> B transaction
            else {
                // increase the block validator's balance by the transaction fee
                match self.accounts.get_mut(&validator_address) {
                    // increase the block validator's balance by the transaction fee
                    Some(account) => account.increase_balance(transaction.fee),
                    None => {
                        // create a new account for this newly discovered address
                        self.accounts.insert(validator_address, Account::new(validator_address));
                        // increase the block validator's balance by the transaction fee
                        self.accounts.get_mut(&validator_address).unwrap().increase_balance(transaction.fee);
                    }
                };
                
                // increase the receipients balance by the transaction amount
                match self.accounts.get_mut(&transaction.recipient) {
                    // increase the receipients balance by the transaction amount
                    Some(account) => account.increase_balance(transaction.amount),
                    None => {
                        // create a new account for this newly discovered address
                        self.accounts.insert(transaction.recipient, Account::new(transaction.recipient));
                        // update the balance of the recipient account
                        self.accounts.get_mut(&transaction.recipient).unwrap().increase_balance(transaction.amount);
                    }
                };

                // decrease the sender balance by the transaction amount + fees, and increase the nonce
                // get the account public key
                let validator_pub_key = match PublicKey::from_sec1_bytes(&transaction.sender) {
                    Ok(validator_pub_key) => validator_pub_key,
                    // This should NEVER happen since this block must have been validated by the verification_engine first
                    Err(_) => return false
                };

                // update the balance of the sender account
                match self.accounts.get_mut(&Wallet::generate_address(&validator_pub_key, true)) {
                    Some(account) => {
                        account.decrease_balance(transaction.amount + transaction.fee);
                        account.increase_nonce();
                    },
                    None => {
                        let new_address = Wallet::generate_address(&validator_pub_key, true);
                        // create a new account for this newly discovered address
                        self.accounts.insert(new_address, Account::new(new_address));
                        // update the balance of the sender account
                        self.accounts.get_mut(&new_address).unwrap().decrease_balance(transaction.amount + transaction.fee);
                        // update the nonce of the sender account
                        self.accounts.get_mut(&new_address).unwrap().increase_nonce();
                    }
                };
            }
        }

        true
    }

    pub fn get_block_height(&self) -> u64 {
        self.block_height
    }

    pub fn get_account(&self, address: &[u8; BLOCK_ADDRESS_SIZE]) -> Option<&Account> {
        self.accounts.get(address)
    }

    pub fn get_last_block(&self) -> &Block {
        self.blocks.last().unwrap()
    }

    pub fn increase_block_height(&mut self) {
        self.block_height += 1;
    }
}