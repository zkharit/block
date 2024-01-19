use k256::ecdsa::{VerifyingKey, signature::Verifier, Signature};

use crate::transaction::{Transaction, TxMetadata};

// ToDo: dont know exactly what return type I should use here
// look into custom error handling Result<(), Err>
pub fn verify_transaction(t: Transaction) -> Result<(), k256::ecdsa::Error> {
    // ToDo: will need to do balance checking when the blockchain/consensus module is created
    // probably need to hold internal state about balances and such so that multiple transactions can be checked in a row including changing account balances
    // check if the account has enough funds to spend

    // compute the TxMetadata struct from the given transaction
    let hashed_serialized_tx_metadata = TxMetadata::serialize_hash_tx_metadata(&TxMetadata::new(t.version, t.amount, t.fee, t.recipient, t.nonce));
    let verifying_key = VerifyingKey::from_sec1_bytes(&t.sender).unwrap();

    // verify the signature and message with the received public key
    verify_sig(&verifying_key, &hashed_serialized_tx_metadata, &t.signature)
}

fn verify_sig(verifying_key: &VerifyingKey, message: &Vec<u8>, signature: &Signature) -> Result<(), k256::ecdsa::Error> {
    verifying_key.verify(message, signature)
}
