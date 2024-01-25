mod config;
mod constants;
mod transaction;
mod wallet;
mod verification_engine;
mod util;

use std::{path::PathBuf, env};

use config::Config;
use transaction::Transaction;
use wallet::Wallet;
use verification_engine::verify_transaction;

use constants::DEFAULT_CONFIG_FILE_NAME;

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

    let mut sending_wallet: Wallet = Wallet::new(config.wallet);
    let receiving_wallet: Wallet = Wallet::new(config_2.wallet);

    println!("Sending Wallet:");
    println!("{:X?}", sending_wallet.get_address());
    println!("");
    println!("Receiving Wallet:");
    println!("{:X?}", receiving_wallet.get_address());
    println!("");

    let tx: Transaction = match sending_wallet.create_tx(200000000, 100000000, receiving_wallet.get_address()) {
        Some(tx) => tx,
        None => {
            println!("Failed to create transaction");
            return
        }
    };
    println!("Transaction:");
    println!("{:X?}", tx);
    println!("");

    let sender_pub_key = sending_wallet.get_public_key().to_sec1_bytes();
    println!("Sender Public Key:");
    println!("{:X?}", sender_pub_key);
    println!("");

    let serialized_tx = tx.serialize_tx();
    println!("Serialized Transaction:");
    println!("{:X?}", serialized_tx);
    println!("");

    let serialized_hashed_tx = tx.serialize_hash_tx();
    println!("Transaction Hash:");
    println!("{:X?}", serialized_hashed_tx);
    println!("");

    let new_tx = Transaction::from(serialized_tx);
    println!("Rebuilt Transaction: ");
    println!("{:X?}", new_tx);
    println!("");

    let transaction_verification_result = verify_transaction(new_tx);
    println!("Transaction Verification:");
    println!("{:?}", transaction_verification_result);
    println!("");

    // read config file
    // generate new wallet/restore wallet
        // save to wallet file/read from wallet file
    // connect to peers
    // sync blockchain
        // validate transactions
        // validate blocks
}
