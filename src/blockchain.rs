use std::collections::HashMap;

use k256::PublicKey;
use sha2::{Sha256, Digest};

use crate::account::Account;
use crate::block::Block;
use crate::transaction::Transaction;
use crate::validator_account::ValidatorAccount;
use crate::verification_engine;
use crate::wallet::Wallet;

use crate::constants::{BLOCK_ADDRESS_SIZE, BOOTSTRAPPING_PHASE_BLOCK_HEIGHT, COMPRESSED_PUBLIC_KEY_SIZE, GENESIS_BLOCK, LOOSE_CHANGE_RECIPIENT, VALIDATOR_ENABLE_RECIPIENT};

#[derive(Debug, Clone)]
pub struct Blockchain {
    // ToDo: limit total blocks stored in memory
    // vector of most recent x blocks
    blocks: Vec<Block>,
    // hashmap of all accounts on the blockchain
    accounts: HashMap<[u8; BLOCK_ADDRESS_SIZE], Account>,
    // vector of all validators on the blockchain
    validators: Vec<ValidatorAccount>,
    // sorted list of unconfirmed transactions by fee
    mempool: Vec<Transaction>,
    // the current blockheight
    block_height: u64
}

impl Blockchain {
    pub fn new() -> Self {
        // create genesis block
        let mut blocks: Vec<Block> = vec![];
        let genesis_block = Block::from(GENESIS_BLOCK.to_vec()).unwrap();

        // add genesis block to chain
        // ToDo: gensis block validation?
        blocks.push(genesis_block.clone());

        // create account set
        let accounts = HashMap::new();

        // create validator set
        let validators: Vec<ValidatorAccount> = vec![];

        // create mempool
        let mempool: Vec<Transaction> = vec![];

        let block_height = 0;

        // create blockchain object
        let mut blockchain = Blockchain {
            blocks,
            accounts,
            validators,
            mempool,
            block_height,
        };

        // update blockchain object with genesis block transaction
        blockchain.update_chain(&genesis_block);

        blockchain
    }

    pub fn add_block(self, block: &Block) -> (bool, Blockchain) {
        // ToDo: can return Option<Blockchain> instead of bool tuple?
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

    pub fn add_transaction_mempool(&mut self, transaction: &Transaction) -> bool {
        // ToDo: need to allow multiple transactions from the same sender to be in the mempool (nonce checking issue)
        // verify the received transaction
        if verification_engine::verify_transaction(transaction, None, self) {
            // if it is valid add it to the mempool
            self.mempool.push(transaction.clone());
            self.mempool.sort();
        } else {
            return false
        }

        true
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
                if verification_engine::is_coinbase(transaction, Some(block), self.get_block_height()) {
                    transaction.recipient
                } else {
                    // if there is no coinbase transaction then the validator will lose all rewards for this block
                    *LOOSE_CHANGE_RECIPIENT
                }
            },
            // if there are no transactions in the block then the validator will lose all rewards for this block
            None => *LOOSE_CHANGE_RECIPIENT
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

            // add the account to the list of validators
            self.validators.push(ValidatorAccount::new(transaction.sender));

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

            // remove the account from the list of validators
            for i in 0..self.validators.len() {
                if self.validators[i].get_public_key() == transaction.sender {
                    self.validators.remove(i);
                    break;
                }
            }

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
            if verification_engine::is_coinbase(&transaction, Some(block), self.get_block_height()) {
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

    pub fn calculate_proposer(&self, validator_list: Vec<ValidatorAccount>, previous_validator_pub_key: Option<[u8; COMPRESSED_PUBLIC_KEY_SIZE]>) -> Option<([u8; COMPRESSED_PUBLIC_KEY_SIZE], usize)> {
        let mut proposer_hash = match previous_validator_pub_key {
            Some(previous_validator_pub_key) => {
                // Note: the previous_validator_pub_key is NOT the previous blocks validator's public key, it is the previous validator that would've been chosen for the CURRENT block
                // this is used in the scenario where a validator didn't propose a block and they were the chosen validator. The previous_validator_pub_key is used as a "seed" for choosing a new validator
                
                // concatenate the previous blocks hash and the previously chosen validator for this block
                let mut concatenated_prev_hash_previous_validator_pub_key: Vec<u8> = vec![];
                concatenated_prev_hash_previous_validator_pub_key.append(&mut self.get_last_block().serialize_hash_block_header());
                concatenated_prev_hash_previous_validator_pub_key.append(&mut previous_validator_pub_key.clone().to_vec());

                // sha256(prev_hash + previous_validator_pub_key)
                let mut sha256_hasher: Sha256 = Sha256::new();
                sha256_hasher.update(concatenated_prev_hash_previous_validator_pub_key);
                sha256_hasher.finalize().to_vec()
            },
            // if this is the first validator (there is no previously attetmpted validator) then just use the previous block hash as the "seed"
            None => self.get_last_block().serialize_hash_block_header()
        };

        // get the bottom 64 bits of the hash
        let bottom_64_bits: Vec<u8> = proposer_hash.drain(24..).collect();

        // convert that to a 64 bit integer
        let bottom_64_as_integer = u64::from_be_bytes(bottom_64_bits.try_into().unwrap());

        // get the total stake the validator_list has staked
        let mut total_stake = 0;

        for i in 0..validator_list.len() {
            let validator_pub_key = match PublicKey::from_sec1_bytes(&validator_list[i].get_public_key()) {
                Ok(validator_pub_key) => validator_pub_key,
                // should never get here
                Err(_) => continue
            };

            // get the address for the account public key
            let account_address = Wallet::generate_address(&validator_pub_key, true);

            let validator_account = match self.get_account(&account_address) {
                Some(validator_account) => validator_account,
                // Should never get here
                None => continue
            };

            // accumulate the total stake variable
            total_stake += validator_account.get_stake();
        }

        // if the blockchain is out of the bootstrapping phase mod the bottom 64 bits integer with the total amount the validator_list has staked
        if self.get_block_height() > *BOOTSTRAPPING_PHASE_BLOCK_HEIGHT {
            let winning_number = bottom_64_as_integer % total_stake;

            // iterate through the validator list (order here matters) and add each stake until youve reached the the winning stake number
            let mut total_staked_accumulation = 0;

            for i in 0..validator_list.len() {
                let validator_pub_key = match PublicKey::from_sec1_bytes(&validator_list[i].get_public_key()) {
                    Ok(validator_pub_key) => validator_pub_key,
                    // should never get here
                    Err(_) => continue
                };

                // get the address for the account public key
                let account_address = Wallet::generate_address(&validator_pub_key, true);

                // get the validator account
                let validator_account = match self.get_account(&account_address) {
                    Some(validator_account) => validator_account,
                    // Should never get here
                    None => continue
                };

                // accumulate the total 
                total_staked_accumulation += validator_account.get_stake();

                // when the winning number has been reached return the winning validator
                if total_staked_accumulation >= winning_number {
                    return Some((validator_list[i].get_public_key(), i))
                }
            }
        } else {
            // if the blockchain is in the bootstrapping phase mod the bottom 64 bits integer with the total amount of validators
            // this prevents a scenario where if 1 validator has staked some coins and none of the others have, the one validator that has staked coins will always be the chosen validator
            // this allows for an attack vector on the network, since you can create unlimited accounts for free (during the bootstrapping phase)
            let winning_number = bottom_64_as_integer % TryInto::<u64>::try_into(validator_list.len()).unwrap();
            return Some((validator_list[winning_number as usize].get_public_key(), winning_number as usize))
        }

        // Should never get here, but in this scenario return none
        None
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

    pub fn get_validators(&self) -> Vec<ValidatorAccount> {
        self.validators.clone()
    }

    pub fn get_mempool(&self) -> Vec<Transaction> {
        self.mempool.clone()
    }

    pub fn remove_from_mempool(&mut self, transaction_count: u64) {
        // removes the first "transaction_count" transactions from the mempool 
        self.mempool = self.mempool.drain(transaction_count as usize..).collect();
    }
}