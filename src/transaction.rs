use k256::ecdsa::Signature;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

use crate::constants::BLOCK_ADDRESS_SIZE;

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    // transaction version
    version: u8,
    // amount in smallest denomination - 0.00000001
    amount:u64,
    // fee in smallest denomination - 0.00000001
    fee: u64,
    // recipeint in address format (base58encoded(version bytes + pubkey + checksum))
    #[serde(with = "serde_big_array::BigArray")]
    recipient: [u8; BLOCK_ADDRESS_SIZE],
    // sender compressed public key 0x02 or 0x03 (if y is even/odd respesctively) + x point
    #[serde(with = "serde_big_array::BigArray")]
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

    pub fn from(raw: Vec<u8>) -> Self {
        bincode::deserialize(&raw).unwrap()
    }

    pub fn serialize_hash_tx(& self) -> Vec<u8> {
        // serialize transaction
        let serialized_tx = self.serialize_tx();
        // sha256(serialized transaction)
        Self::hash_serialized_tx(serialized_tx)
    }

    pub fn serialize_tx(& self) -> Vec<u8> {
        // ToDo: serialized in little endian order, maybe should change to big endian with bincode::Config/bincode::Options/bincode::DefaultOptions
        bincode::serialize(self).unwrap()
    }

    fn hash_serialized_tx(serialized_tx: Vec<u8>) -> Vec<u8> {
        // sha256(serialized transaction metadata)
        let mut sha256_hasher: Sha256 = Sha256::new();
        sha256_hasher.update(serialized_tx);
        sha256_hasher.finalize().to_vec()
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

    pub fn serialize_hash_tx_metadata(& self) -> Vec<u8> {
        // serialize transaction metadata
        let serialized_tx_metadata = self.serialize_tx_metadata();
        // sha256(serialized transaction metadata)
        Self::hash_serialized_tx_metadata(serialized_tx_metadata)
    }

    pub fn serialize_tx_metadata(& self) -> Vec<u8> {
        // ToDo: serialized in little endian order, maybe should change to big endian with bincode::Config/bincode::Options/bincode::DefaultOptions
        bincode::serialize(self).unwrap()
    }

    fn hash_serialized_tx_metadata(serialized_tx_metadata: Vec<u8>) -> Vec<u8> {
        // sha256(serialized transaction metadata)
        let mut sha256_hasher: Sha256 = Sha256::new();
        sha256_hasher.update(serialized_tx_metadata);
        sha256_hasher.finalize().to_vec()
    }
}