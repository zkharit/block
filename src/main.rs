mod account;
mod block;
mod blockchain;
mod config;
mod constants;
mod transaction;
mod wallet;
mod validator;
mod validator_account;
mod verification_engine;
mod util;

use std::{path::PathBuf, env};

use block::Block;
use blockchain::Blockchain;
use config::Config;
use transaction::Transaction;
use wallet::Wallet;
use validator::Validator;
use verification_engine::verify_transaction;

use constants::{DEFAULT_CONFIG_FILE_NAME, BLOCK_VERSION};

fn main() {
    // Test Vectors:
    // Test secret key found on: https://en.bitcoin.it/wiki/Technical_background_of_version_1_Bitcoin_addresses#How_to_create_Bitcoin_Address
    // let secret_key_material: &[u8] = &[24, 225, 74, 123, 106, 48, 127, 66, 106, 148, 248, 17, 71, 1, 231, 200, 231, 116, 231, 249, 164, 126, 44, 32, 53, 219, 41, 162, 6, 50, 23, 37];

    // Test secret key found on: https://en.bitcoin.it/wiki/Wallet_import_format
    // let secret_key_material: &[u8] = &[12, 40, 252, 163, 134, 199, 162, 39, 96, 11, 47, 229, 11, 124, 174, 17, 236, 134, 211, 191, 31, 190, 71, 27, 232, 152, 39, 225, 157, 114, 170, 29];

    // let secret_key_material: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];

    // This is 2**256 - 2**32 - 2**9 - 2**8 - 2**7 - 2**6 - 2**4 - 1 which I believe is supposed to be the max number in ecdsa, but this breaks when attempting to generate a key from it (or minus 1)
    // let secret_key_material: &[u8] = &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 255, 255, 252, 47];

    // used in bitcoin-core/secp256k1 ecdsa_impl.h for some reason? which also breaks when attempting to generate a key from it, BUT 1 less than it seems to work, so that must be the max value for sec256kp1?
    // let secret_key_material: &[u8] = &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 186, 174, 220, 230, 175, 72, 160, 59, 191, 210, 94, 140, 208, 54, 65, 65];
    // let secret_key_material: &[u8] = &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 186, 174, 220, 230, 175, 72, 160, 59, 191, 210, 94, 140, 208, 54, 65, 64];
    
    // let secret_key = SecretKey::from_bytes(secret_key_material.into()).unwrap();

    let args: Vec<String> = env::args().collect();
    let config: Config;
    if args.len() == 2 { 
        config = Config::new(&PathBuf::from(&args[1]));
    } else {
        config = Config::new(&PathBuf::from(DEFAULT_CONFIG_FILE_NAME));
    }

    // use a different config object but same config as the sending wallet just for testing purposes
    let config_2 = Config::new(&PathBuf::from(DEFAULT_CONFIG_FILE_NAME));

    println!("Wallet File Name (from config):");
    println!("{}", config.get_wallet_config().get_wallet_file().display());
    println!("");

    let mut sending_wallet: Wallet = Wallet::new(config.get_wallet_config());
    let receiving_wallet: Wallet = Wallet::new(config_2.get_wallet_config());

    println!("Sending Wallet:");
    println!("{:X?}", sending_wallet.get_address());
    println!("");
    println!("Receiving Wallet:");
    println!("{:X?}", receiving_wallet.get_address());
    println!("");

    // let tx: Transaction = match sending_wallet.create_tx(200000000, 100000000, receiving_wallet.get_address()) {
    //     Some(tx) => tx,
    //     None => {
    //         println!("Failed to create transaction");
    //         return
    //     }
    // };
    // println!("Transaction:");
    // println!("{:X?}", tx);
    // println!("");

    // let sender_pub_key = sending_wallet.get_public_key().to_sec1_bytes();
    // println!("Sender Public Key:");
    // println!("{:X?}", sender_pub_key);
    // println!("");

    // let serialized_tx = tx.serialize_tx();
    // println!("Serialized Transaction:");
    // println!("{:X?}", serialized_tx);
    // println!("");

    // let serialized_hashed_tx = tx.serialize_hash_tx();
    // println!("Transaction Hash:");
    // println!("{:X?}", serialized_hashed_tx);
    // println!("");

    // let new_tx = match Transaction::from(serialized_tx) {
    //     Ok(tx) => tx,
    //     Err(_) => {
    //         panic!("Failed rebuilding serialized transaction");
    //     }
    // };
    // println!("Rebuilt Transaction: ");
    // println!("{:X?}", new_tx);
    // println!("");

    // let transaction_verification_result = verify_transaction(new_tx);
    // println!("Transaction Verification:");
    // println!("{:?}", transaction_verification_result);
    // println!("");

    // let tx1: Transaction = match sending_wallet.create_tx(200000000, 100000000, receiving_wallet.get_address()) {
    //     Some(tx) => tx,
    //     None => {
    //         println!("Failed to create transaction");
    //         return
    //     }
    // };

    // let tx2: Transaction = match sending_wallet.create_tx(200000000, 100000000, receiving_wallet.get_address()) {
    //     Some(tx) => tx,
    //     None => {
    //         println!("Failed to create transaction");
    //         return
    //     }
    // };

    // let tx3: Transaction = match sending_wallet.create_tx(400000000, 100000000, receiving_wallet.get_address()) {
    //     Some(tx) => tx,
    //     None => {
    //         println!("Failed to create transaction");
    //         return
    //     }
    // };

    // let tx4: Transaction = match sending_wallet.create_tx(400000000, 200000000, receiving_wallet.get_address()) {
    //     Some(tx) => tx,
    //     None => {
    //         println!("Failed to create transaction");
    //         return
    //     }
    // };

    // let tx5: Transaction = match sending_wallet.create_tx(500000000, 200000000, receiving_wallet.get_address()) {
    //     Some(tx) => tx,
    //     None => {
    //         println!("Failed to create transaction");
    //         return
    //     }
    // };

    // let tx6: Transaction = match sending_wallet.create_tx(600000000, 200000000, receiving_wallet.get_address()) {
    //     Some(tx) => tx,
    //     None => {
    //         println!("Failed to create transaction");
    //         return
    //     }
    // };

    // println!("Tx1 Hash: {:?}", tx1.serialize_hash_tx());
    // println!("Tx2 Hash: {:?}", tx2.serialize_hash_tx());
    // println!("Tx3 Hash: {:?}", tx3.serialize_hash_tx());
    // println!("Tx4 Hash: {:?}", tx4.serialize_hash_tx());
    // println!("Tx5 Hash: {:?}", tx5.serialize_hash_tx());
    // println!("Tx6 Hash: {:?}", tx6.serialize_hash_tx());

    // let tx_vec = vec![tx1.clone(), tx2.clone(), tx3.clone(), tx4.clone(), tx5.clone(), tx6.clone()];
    // let block1 = Block::new(0x01, [0x00; 32], 0x02, tx_vec);

    // let tx_vec2 = vec![tx1.clone(), tx2, tx3, tx4, tx5.clone(), tx6.clone(), tx5, tx6];
    // let block2 = Block::new(0x01, [0x00; 32], 0x02, tx_vec2);

    // let tx_vec3 = vec![tx1.clone()];
    // let block3 = Block::new(0x01, [0x00; 32], 0x02, tx_vec3);
    

    // let serialized_block1 = block1.serialize_block();
    // println!("Serialized Block 1:");
    // println!("{:X?}", serialized_block1);
    // println!("");

    // let new_block1 = Block::from(serialized_block1);
    // println!("Rebuilt Block 1:");
    // println!("{:X?}", new_block1);
    // println!("");

    // let serialized_block2 = block2.serialize_block();
    // println!("Serialized Block 2:");
    // println!("{:X?}", serialized_block2);
    // println!("");

    // let new_block2 = Block::from(serialized_block2);
    // println!("Rebuilt Block 2:");
    // println!("{:X?}", new_block2);
    // println!("");

    // let serialized_block3 = block3.serialize_block();
    // println!("Serialized Block 3:");
    // println!("{:X?}", serialized_block3);
    // println!("");

    // let new_block3 = Block::from(serialized_block3.clone());
    // println!("Rebuilt Block 3:");
    // println!("{:X?}", new_block3);
    // println!("");

    let mut validator = Validator::new(config.get_validator_config(), &sending_wallet);
    let coinbase_tx = validator.create_coinbase_tx(0).unwrap();
    println!("Coinbase TX:");
    println!("{:X?}", coinbase_tx);
    println!("");

    println!("Serialized Coinbase TX:");
    println!("{:X?}", coinbase_tx.serialize_tx());
    println!("");

    println!("Serialized Hashed Coinbase TX:");
    println!("{:X?}", coinbase_tx.serialize_hash_tx());
    println!("");

    let gensis_txs = vec![coinbase_tx];
    let genesis_block = Block::new(*BLOCK_VERSION, [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 123, &gensis_txs, sending_wallet.create_block_sig(*BLOCK_VERSION, [0x00; 32], 123, &gensis_txs).unwrap());
    println!("Genesis Block:");
    println!("{:X?}", genesis_block);
    println!("");

    println!("Serialized Genesis Block:");
    println!("{:X?}", genesis_block.serialize_block());
    println!("");

    let blockchain = Blockchain::new();

    let validator_enable_tx = sending_wallet.create_validator_enable_tx(50, 500).unwrap();

    let validator_tx_vec = vec![validator_enable_tx];
    let validator_block = Block::new(*BLOCK_VERSION, blockchain.get_last_block().serialize_hash_block_header().try_into().unwrap(), 0x02, &validator_tx_vec, sending_wallet.create_block_sig(*BLOCK_VERSION, blockchain.get_last_block().serialize_hash_block_header().try_into().unwrap(), 0x02, &validator_tx_vec).unwrap());

    let (result, blockchain) = blockchain.add_block(&validator_block);

    if !result {
        println!("Blockchain failed to update due to invalid transaction or block");
    }

    println!("Blockchain:");
    println!("{:X?}", blockchain);
    println!("");

    let validator_revoke_tx = sending_wallet.create_validator_revoke_tx(50, 500).unwrap();

    let validator_tx_vec_2 = vec![validator_revoke_tx];
    let validator_revoke_block = Block::new(*BLOCK_VERSION, blockchain.get_last_block().serialize_hash_block_header().try_into().unwrap(), 0x02, &validator_tx_vec_2, sending_wallet.create_block_sig(*BLOCK_VERSION, blockchain.get_last_block().serialize_hash_block_header().try_into().unwrap(), 0x02, &validator_tx_vec_2).unwrap());

    let (result, blockchain) = blockchain.add_block(&validator_revoke_block);

    if !result {
        println!("Blockchain failed to update due to invalid transaction or block");
    }

    println!("Blockchain:");
    println!("{:X?}", blockchain);
    println!("");

    // read config file
    // generate new wallet/restore wallet
        // save to wallet file/read from wallet file
    // connect to peers
    // sync blockchain
        // validate transactions
        // validate blocks
}
