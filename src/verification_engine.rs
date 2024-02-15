use k256::ecdsa::{VerifyingKey, signature::Verifier, Signature};
use k256::PublicKey;

use crate::block::{Block, BlockHeader};
use crate::blockchain::Blockchain;
use crate::transaction::{Transaction, TxMetadata};
use crate::wallet::Wallet;

use crate::constants::{BOOTSTRAPPING_PHASE_BLOCK_HEIGHT, COINBASE_SENDER, HALVING_INTERVAL, LOWEST_DENOMINATION_PER_COIN, MINIMUM_STAKING_AMOUNT, VALIDATOR_ENABLE_RECIPIENT, VALIDATOR_REVOKE_RECIPIENT};

pub fn verify_transaction(transaction: &Transaction, block: &Block, blockchain: &Blockchain) -> bool {
    // compute the TxMetadata struct from the given transaction
    let hashed_serialized_tx_metadata = TxMetadata::serialize_hash_tx_metadata(&TxMetadata::new(transaction.version, transaction.amount, transaction.fee, transaction.recipient, transaction.nonce));
    let verifying_key = match VerifyingKey::from_sec1_bytes(&transaction.sender) {
        Ok(verifying_key) => verifying_key,
        // if an invalid public key is received then the transaction is invalid
        Err(_) => return false
    };

    // verify the signature and message with the received public key
    if !verify_sig(&verifying_key, &hashed_serialized_tx_metadata, &transaction.signature) {
        return false
    }

    // get the account public key
    let account_pub_key = match PublicKey::from_sec1_bytes(&transaction.sender) {
        Ok(account_pub_key) => account_pub_key,
        // if an invalid public key is received then the transaction is invalid
        Err(_) => return false
    };

    // get the address for the account public key
    let account_address = Wallet::generate_address(&account_pub_key, true);

    // confirm the nonce is correct for this transaction unless it is a coinbase transaction
    if !is_coinbase(&transaction, &block, blockchain.get_block_height()) {
        // obtain the account nonce in the blockchains view
        let tx_account_nonce = match blockchain.get_account(&account_address) {
            Some(tx_account) => {
                tx_account.get_nonce()
            }
            None => {
                // if the account isn't in the blockchain yet, then the nonce needs to be 0
                0
            }
        };

        // make sure the account nonce in view of the blockchain is the same as the transaction nonce 
        if transaction.nonce != tx_account_nonce {
            return false
        }
    }
    
    // check for proper balances
    if is_validator_enable(&transaction, &blockchain) {
        // ToDo: what else do I need to check here?
        // ToDo: confirm the sender is the proper person?

        // obtain the sender balance
        let account_balance = match blockchain.get_account(&account_address) {
            Some(tx_account) => {
                tx_account.get_balance()
            },
            // if the account is not within the blockchain then it has no funds, but it can send a validator enable tx with 0 fees and 0 stake (during the bootstrapping phase)
            None => 0
        };

        // confirm the sender's balance is at least the transaction fee
        if account_balance < (transaction.fee + transaction.amount) {
            return false
        }
    } else if is_validator_revoke(&transaction, &blockchain) {
        // ToDo: what else do I need to check here?
        // ToDo: confirm the sender is the proper person?

        // obtain the sender balance
        let account_balance = match blockchain.get_account(&account_address) {
            Some(tx_account) => {
                tx_account.get_balance()
            },
            // if the account is not within the blockchain then it has no funds, but it can send a validator revoke tx with 0 fees, technically should never be able to get here
            None => 0
        };

        // confirm the sender's balance is at least the transaction fee
        if account_balance < transaction.fee {
            return false
        }
    } else if is_coinbase(&transaction, &block, blockchain.get_block_height()) {
        // ToDo: what else do I need to check here?
        // ToDo: confirm the sender is the proper person?
    } else {
        // obtain the sender balance
        let account_balance = match blockchain.get_account(&account_address) {
            Some(tx_account) => {
                tx_account.get_balance()
            },
            // if the account is not within the blockchain then it definitely doesn't have sufficient funds
            None => return false
        };

        // confirm the sender's balance is at least the transaction amount and transaction fee
        if account_balance < (transaction.fee + transaction.amount) {
            return false
        }
    }

    true
}

fn verify_sig(verifying_key: &VerifyingKey, message: &Vec<u8>, signature: &Signature) -> bool {
    match verifying_key.verify(message, signature) {
        Ok(_) => return true,
        Err(_) => return false
    }
}

pub fn verify_block(block: Block, blockchain: &Blockchain) -> bool {
    // ToDo: need to include a block signature signed by the proposer and validate that signature

    // temporary blockchain to maintain state within the block and its transactions
    let mut new_blockchain = blockchain.clone();

    // confirm the proposed block previous hash matches the blockchains last blocks previous hash
    if new_blockchain.get_last_block().serialize_hash_block_header() != block.prev_hash() {
        return false
    }

    // calculate the merkle root of the transactions received in the block
    let merkle_root = BlockHeader::calculate_merkle_root(block.get_transactions().clone());
    
    // confirm the merkle root of the block matches that of the transaction list
    if merkle_root != block.merkle_root() {
        return false
    }

    // ToDo: confirm if the correct validator proposed this block with the block signature

    // ToDo: confirm the timestamp of the proposed block matches the systemtime current timestamp (wtihin some interval)

    for transaction in block.get_transactions() {
        // verify each transaction and update the local copy of the blockchain
        if verify_transaction(transaction, &block, &new_blockchain) {
            if !new_blockchain.update_chain_transaction(transaction, &block) {
                return false
            }
        } else {
            return false
        }
    }

    true
}

pub fn is_coinbase(transaction: &Transaction, block: &Block, block_height: u64) -> bool {
    // the signature of a coinbase transaction only needs to be a valid signature, its contents are never checked

    // make sure the transactions is the first transaction in the block
    match block.get_transactions().get(0) {
        Some(tx) => {
            if *transaction != *tx {
                return false
            }
        },
        None => return false
    }

    // confirm the sender is the standard COINBASE_SENDER public key
    if transaction.sender != *COINBASE_SENDER {
        return false;
    }

    // confirm the transaction fee is 0
    if transaction.fee != 0 {
        return false;
    }

    // confirm the transaction nonce is 0
    if transaction.nonce != 0 {
        return false;
    }

    // max block reward for the next block
    let max_block_reward = get_block_subsidy(block_height + 1);

    // make sure the reward amount is less than the maximum reward amount for the current blockheight
    if transaction.amount > max_block_reward {
        return false
    }

    true
}

pub fn is_validator_enable(transaction: &Transaction, blockchain: &Blockchain) -> bool {
    // get the public key from the transaction
    let validator_pub_key = match PublicKey::from_sec1_bytes(&transaction.sender) {
        Ok(validator_pub_key) => validator_pub_key,
        Err(_) => return false
    };

    // get the account address
    let validator_address = Wallet::generate_address(&validator_pub_key, true);

    // confirm user is not already a validator on chain
    match blockchain.get_account(&validator_address) {
        Some(validator_account) =>  {
            if validator_account.get_validator() {
                return false
            }
            ()
        },
        None => ()
    };
    
    // confirm the recipient is the standard VALIDATOR_ENABLE_RECIPIENT address
    if transaction.recipient != *VALIDATOR_ENABLE_RECIPIENT {
        return false
    }

    // confirm they user has input the minimum amount to stake considering the boostrapping phase
    if blockchain.get_block_height() >= *BOOTSTRAPPING_PHASE_BLOCK_HEIGHT {
        if transaction.amount < *MINIMUM_STAKING_AMOUNT {
            return false
        }
    }

    true
}

pub fn is_validator_revoke(transaction: &Transaction, blockchain: &Blockchain) -> bool {
    // get the public key from the transaction
    let validator_pub_key = match PublicKey::from_sec1_bytes(&transaction.sender) {
        Ok(validator_pub_key) => validator_pub_key,
        Err(_) => return false
    };

    // get the account address
    let validator_address = Wallet::generate_address(&validator_pub_key, true);

    // confirm user is already a validator on chain
    let validator_account = match blockchain.get_account(&validator_address) {
        Some(validator_account) =>  {
            if !validator_account.get_validator() {
                return false
            }
            validator_account
        },
        None => return false
    };

    // confirm the recipient is the standard VALIDATOR_REVOKE_RECIPIENT address
    if transaction.recipient != *VALIDATOR_REVOKE_RECIPIENT {
        return false;
    }

    // confirm the amount the validator is attempting to revoke matches the amount they have staked
    if transaction.amount != validator_account.get_stake() {
        return false;
    }

    true
}

pub fn get_block_subsidy(block_height: u64) -> u64 {
    // taken straight from bitcoin's codebase : )
    let halvings = block_height / HALVING_INTERVAL;

    if halvings >= 64 {
        return 0
    }

    let mut block_reward = 50 * LOWEST_DENOMINATION_PER_COIN;

    block_reward >>= halvings;

    block_reward
}
