use k256::ecdsa::{VerifyingKey, signature::Verifier, Signature};
use k256::PublicKey;

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::transaction::{Transaction, TxMetadata};
use crate::wallet::Wallet;

use crate::constants::{BOOTSTRAPPING_PHASE_BLOCK_HEIGHT, COINBASE_SENDER, HALVING_INTERVAL, LOWEST_DENOMINATION_PER_COIN, MINIMUM_STAKING_AMOUNT, VALIDATOR_ENABLE_RECIPIENT, VALIDATOR_REVOKE_SENDER};

// ToDo: dont know exactly what return type I should use here
// look into custom error handling Result<(), Err>
pub fn verify_transaction(t: Transaction) -> Result<(), k256::ecdsa::Error> {
    // ToDo: will need to do balance checking when the blockchain/consensus module is created
    // probably need to hold internal state about balances and such so that multiple transactions can be checked in a row including changing account balances
    // check if the account has enough funds to spend

    // ToDo: this will not work for validator_revoke transactions at the moment, because the sender for those transdactions
    // is the VALIDATOR_REVOKE_SENDER whose private key did not sign this transaction, the receipient's public key is the signer
    // How to obtain the public key from the recipient address? <- not possible Need to think about how this type of transaction can be verified

    // compute the TxMetadata struct from the given transaction
    let hashed_serialized_tx_metadata = TxMetadata::serialize_hash_tx_metadata(&TxMetadata::new(t.version, t.amount, t.fee, t.recipient, t.nonce));
    let verifying_key = VerifyingKey::from_sec1_bytes(&t.sender).unwrap();

    // verify the signature and message with the received public key
    verify_sig(&verifying_key, &hashed_serialized_tx_metadata, &t.signature)
}

fn verify_sig(verifying_key: &VerifyingKey, message: &Vec<u8>, signature: &Signature) -> Result<(), k256::ecdsa::Error> {
    verifying_key.verify(message, signature)
}

pub fn verify_block(block: &Block, blockchain: &Blockchain) -> bool {
    // ToDo: 
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
    // get the account address
    let validator_address = &transaction.recipient;

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

    // confirm the sender is the standard VALIDATOR_REVOKE_RECIPIENT address
    if transaction.sender != *VALIDATOR_REVOKE_SENDER {
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
