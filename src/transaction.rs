use k256::ecdsa::Signature;
use serde::{Serialize, Deserialize};

use crate::constants::BLOCK_ADDRESS_SIZE;

#[derive(Debug)]
pub struct Transaction {
    // transaction version
    version: u8,
    // amount in smallest denomination - 0.00000001
    amount:u64,
    // fee in smallest denomination - 0.00000001
    fee: u64,
    // recipeint in address format (base58encoded(version bytes + pubkey + checksum))
    recipient: [u8; BLOCK_ADDRESS_SIZE],
    // sender compressed public key 0x02 or 0x03 (if y is even/odd respesctively) + x point
    sender: [u8; BLOCK_ADDRESS_SIZE],
    // sign(sha256(version + amount + fee + recipient + nonce))
    signature: Signature,
    // account nonce, incremented once for each confirmed transaction
    nonce: u64,
    // sha256(version + amount + fee + recipient + nonce)
    // hash: Vec<u8> Not sure if this field is necessary (might just be needed to be included in the blockchain)
}

impl Transaction {
    pub fn new(version: u8, amount: u64, fee: u64, recipient: [u8; BLOCK_ADDRESS_SIZE], sender: [u8; BLOCK_ADDRESS_SIZE], signature: Signature, nonce: u64) -> Self {
        Self {
            version,
            amount,
            fee,
            recipient,
            sender,
            signature,
            nonce
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TxMetadata {
    version: u8,
    amount: u64,
    fee: u64,
    #[serde(with = "serde_big_array::BigArray")]
    recipient: [u8; 39],
    nonce: u64,
}

impl TxMetadata {
    pub fn new(version: u8, amount: u64, fee: u64, recipient: [u8; 39], nonce: u64) -> Self {
        Self {
            version,
            amount,
            fee,
            recipient,
            nonce,
        }
    }
}