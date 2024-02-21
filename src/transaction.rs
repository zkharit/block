use std::cmp::Ordering;

use k256::ecdsa::Signature;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use bincode::{Options, ErrorKind};

use crate::constants::{BLOCK_ADDRESS_SIZE, COMPRESSED_PUBLIC_KEY_SIZE};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    // transaction version
    pub version: u8,
    // amount in smallest denomination - 0.00000001
    pub amount:u64,
    // fee in smallest denomination - 0.00000001
    pub fee: u64,
    // recipeint in address format (base58encoded(version bytes + pubkey + checksum))
    #[serde(with = "serde_big_array::BigArray")]
    pub recipient: [u8; BLOCK_ADDRESS_SIZE],
    // sender compressed public key 0x02 or 0x03 (if y is even/odd respesctively) + x point
    #[serde(with = "serde_big_array::BigArray")]
    pub sender: [u8; COMPRESSED_PUBLIC_KEY_SIZE],
    // sign(sha256(version + amount + fee + recipient + nonce))
    pub signature: Signature,
    // account nonce, incremented once for each confirmed transaction
    pub nonce: u64,
}

impl Transaction {
    pub fn new(version: u8, amount: u64, fee: u64, recipient: [u8; BLOCK_ADDRESS_SIZE], sender: [u8; COMPRESSED_PUBLIC_KEY_SIZE], signature: Signature, nonce: u64) -> Self {
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

    pub fn from(raw: Vec<u8>) -> Result<Self, Box<ErrorKind>> {
        bincode::DefaultOptions::new()
            .allow_trailing_bytes()
            .with_fixint_encoding()
            .with_big_endian()
            .deserialize(&raw)
    }

    pub fn serialize_hash_tx(& self) -> Vec<u8> {
        // serialize transaction
        let serialized_tx = self.serialize_tx();
        // sha256(serialized transaction)
        Self::hash_serialized_tx(serialized_tx)
    }

    pub fn serialize_tx(& self) -> Vec<u8> {
        bincode::DefaultOptions::new()
            .allow_trailing_bytes()
            .with_fixint_encoding()
            .with_big_endian()
            .serialize(self).unwrap()
    }

    fn hash_serialized_tx(serialized_tx: Vec<u8>) -> Vec<u8> {
        // sha256(serialized transaction)
        let mut sha256_hasher: Sha256 = Sha256::new();
        sha256_hasher.update(serialized_tx);
        sha256_hasher.finalize().to_vec()
    }
}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (other.fee).cmp(&self.fee)
    }
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.fee.cmp(&self.fee))
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        (self.version, self.amount, self.fee, self.recipient, self.sender, self.signature, self.nonce) == 
        (other.version, other.amount, other.fee, other.recipient, other.sender, other.signature, other.nonce)
    }
}

impl Eq for Transaction {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxMetadata {
    pub version: u8,
    pub amount: u64,
    pub fee: u64,
    #[serde(with = "serde_big_array::BigArray")]
    pub recipient: [u8; 39],
    pub nonce: u64,
}

impl TxMetadata {
    pub fn new(version: u8, amount: u64, fee: u64, recipient: [u8; BLOCK_ADDRESS_SIZE], nonce: u64) -> Self {
        Self {
            version,
            amount,
            fee,
            recipient,
            nonce,
        }
    }

    pub fn serialize_hash_tx_metadata(& self) -> Vec<u8> {
        // serialize transaction metadata
        let serialized_tx_metadata = self.serialize_tx_metadata();
        // sha256(serialized transaction metadata)
        Self::hash_serialized_tx_metadata(serialized_tx_metadata)
    }

    pub fn serialize_tx_metadata(& self) -> Vec<u8> {
        bincode::DefaultOptions::new()
            .allow_trailing_bytes()
            .with_fixint_encoding()
            .with_big_endian()
            .serialize(self).unwrap()
    }

    fn hash_serialized_tx_metadata(serialized_tx_metadata: Vec<u8>) -> Vec<u8> {
        // sha256(serialized transaction metadata)
        let mut sha256_hasher: Sha256 = Sha256::new();
        sha256_hasher.update(serialized_tx_metadata);
        sha256_hasher.finalize().to_vec()
    }
}