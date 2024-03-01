mod account;
mod block;
mod blockchain;
mod config;
mod constants;
mod controller;
mod transaction;
mod wallet;
mod validator;
mod validator_account;
mod verification_engine;
mod util;

use std::io::{self, Write};

use crate::controller::Controller;

use constants::{BLOCK_ADDRESS_SIZE, LOWEST_DENOMINATION_PER_COIN, NODE_VERSION};

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

    // For generating new genesis block
    // let coinbase_tx = validator.create_coinbase_tx(0).unwrap();
    // let validator_enable_tx = wallet.create_validator_enable_tx(0, 0).unwrap();
    // let genesis_txs = vec![coinbase_tx, validator_enable_tx.clone()];

    // let genesis_sig = wallet.create_block_sig(*BLOCK_VERSION, [0x00; 32], 0x01, &genesis_txs).unwrap();

    // let genesis_block = Block::new(*BLOCK_VERSION, [0x00;32], 0x01, &genesis_txs, genesis_sig);

    // println!("Genesis Block: {:X?}", genesis_block);

    // println!("Serialized genesis block: {:X?}", genesis_block.serialize_block());

    println!("Welcome to block");
    println!("Node version: {}", NODE_VERSION);

    println!("Initializing node");
    println!();
    let mut controller = Controller::new();
    println!("Initialized node");
    println!();

    // after node has been initialized output options to the user
    output_options(&mut controller);

    println!("Thanks for being apart of the block network!");
    println!("Goodbye!");
}

fn output_options(controller: &mut Controller) {
    let primary_options = vec!["Options:", "Wallet", "Blockchain", "Transaction", "About", "Exit"];

    loop {
        // present high level options to user
        print_options(&primary_options);

        // get user choice
        let option_input = read_string().to_lowercase();
        println!();

        match option_input.as_str() {
            "1" | "1." | "wallet" => {
                perform_wallet_options(controller);
            },
            "2" | "2." | "blockchain" => {
                perform_blockhain_options(controller);
            },
            "3" | "3." | "transaction" => {
                perform_transaction_options(controller);
            },
            "4" | "4." | "about" => {
                perform_about_options(controller);
            },
            "5" | "5." | "exit" => {
                println!("Are you sure you want to exit, you will stop receiving blocks and transactions from other nodes in the network and you will need to re-sync your blockchain next time you start your node? (yes/no)");
                let final_input = read_string().to_lowercase();
                println!();

                match final_input.as_str() {
                    "yes" => break,
                    _ => continue
                }
            }
            _ => continue
        };
    }
}

fn perform_wallet_options(controller: &mut Controller) {
    let wallet_options = vec!["Wallet Options:", "View Overview", "View address", "View balance", "View nonce", "View private key", "Set nonce", "Back"];

    loop {
        // present wallet options to user
        print_options(&wallet_options);

        // get user choice
        let option_input = read_string().to_lowercase();
        println!();

        match option_input.as_str() {
            "1" | "1." | "view overview" | "overview" => {
                println!("Overview:");
                controller.wallet_overview();
                println!();
            },
            "2" | "2." | "view address" | "address" => {
                println!("Address: {}", controller.wallet_get_address_string());
                println!();
            },
            "3" | "3." | "view balance" | "balance" => {
                println!("Balance: {:.8} BLO", controller.wallet_get_balance());
                println!();
            },
            "4" | "4." | "view nonce" => {
                println!("Nonce: {}", controller.wallet_get_nonce());
                println!();
            },
            "5" | "5." | "view private key" | "private key" | "priv key"  | "privkey" => {
                // confirm with user that sensitive data is about to be displayed
                println!("This is sensitive information that no one should view besides yourself, please enter any character when you are ready");
                read_string();
                // obtain the user private key and display if it is obtained
                match controller.wallet_get_private_key() {
                    Some(private_key) => println!("Private Key (WIF): {}", private_key),
                    None => println!("Unable to retreive private key from wallet file")
                }
                
                println!();
            },
            "6" | "6." | "set nonce" => {
                loop {
                    // display the current wallet nonce and prompt the user for the new nonce
                    println!("Current nonce: {}", controller.wallet_get_nonce());
                    println!("Enter the nonce to set:");
                    let new_nonce = match read_string().parse::<u64>() {
                        Ok(new_nonce) => new_nonce,
                        Err(_) => {
                            println!("Enter a valid non-negative nonce");
                            println!();
                            continue
                        }
                    };
                    
                    controller.wallet_set_nonce(new_nonce);
                    println!();
                    println!("Set nonce to {}", new_nonce);
                    println!();
                    break;
                }
            },
            "7" | "7." | "back" => {
                break;
            },
            _ => {}
        }
    }
}

fn perform_blockhain_options(controller: &mut Controller) {
    let blockchain_options = vec!["Blockchain Options:", "View Overview", "View block height", "View block", "View transaction", "View address", "View mempool", "View validators", "View total staked", "View total loose change", "Prune mempool", "Back"];

    loop {
        // present blockchain options to user
        print_options(&blockchain_options);

        // get user choice
        let option_input = read_string().to_lowercase();
        println!();

        match option_input.as_str() {
            "1" | "1." | "view overview" | "overview" => {
                println!("Overview:");
                controller.blockchain_overview();
                println!();
            },
            "2" | "2." | "view block height" | "block height" |  "blockheight" => {
                println!("Current block height: {}", controller.blockchain_get_block_height());
                println!();
            },
            "3" | "3." | "view block" | "block" => {
                loop {
                    // prompt the user for the block height of the block theyd like to view
                    println!("Enter the block height:");
                    let block_height = match read_string().parse::<u64>() {
                        Ok(block_height) => block_height,
                        Err(_) => {
                            println!("Enter a valid non-negative block height");
                            println!();
                            continue
                        }
                    };

                    // if the block height is too high re-ask the user
                    if block_height > controller.blockchain_get_block_height() {
                        println!();
                        println!("Max block height: {}", controller.blockchain_get_block_height());
                        continue;
                    }
                    
                    println!();
                    println!("{:X?}", controller.blockchain_get_block(block_height).unwrap());
                    println!();

                    break;
                }
            },
            "4" | "4." | "view transaction" | "transaction" => {
                // ToDo:
                loop {
                    println!("Not yet implemented");
                    println!();
                    // println!("Enter the transaction hash:");
                    // let tx_hash = read_string().to_lowercase();

                    // if !validate_tx_hash(tx_hash) {
                    //     println!("Invalid transaction hash: {}", tx_hash);
                    //     continue;
                    // }
                    
                    // println!();
                    // println!("{:X?}", controller.blockchain_get_transaction(tx_hash).unwrap());
                    // println!();

                    break;
                }
            },
            "5" | "5." | "view address" | "address" => {
                loop {
                    // prompt the user for the address theyd like to view
                    println!("Enter the address (case sensitive) or \"exit\":");
                    let address_string = read_string();
                    println!();

                    // exit if they entered exit
                    if address_string.to_lowercase() == "exit" {
                        println!();
                        break;
                    }

                    // attempt to convert the address string to a vector
                    let address_arr: [u8; 39] = match address_string.as_bytes().try_into() {
                        Ok(address ) => address,
                        Err(_) => {
                            println!("Invalid address");
                            println!();
                            continue;
                        }
                    };

                    // address checksum checking
                    if !controller.check_address_checksum(address_arr) {
                        println!("Invalid address, please check for typos");
                        println!();
                        continue;
                    }

                    // get the account from the blockchain
                    match controller.blockchain_get_account(&address_arr) {
                        Some(account) => println!("{:X?}", account),
                        None => println!("Address not found on the blockchain")
                    }

                    println!();
                    break;
                }
            },
            "6" | "6." | "view mempool" | "mempool" => {
                println!("Mempool:");
                println!("{:X?}", controller.blockchain_get_mempool());
                println!();
            },
            "7" | "7." | "view validators"  | "validators" => {
                println!("Validators:");
                println!("{:X?}", controller.blockchain_get_validators());
                println!();
            },
            "8" | "8." | "view total staked"  | "total staked" | "staked" => {
                println!("Total staked on block: {:.8} BLO", controller.blockchain_get_total_staked() as f64 / LOWEST_DENOMINATION_PER_COIN);
                println!();
            },
            "9" | "9." | "view total loose change"  | "view loose change" | "loose change" | "change" => {
                println!("Total loose change: {:.8} BLO", controller.blockchain_get_total_change() as f64 / LOWEST_DENOMINATION_PER_COIN);
                println!();
            },
            "10" | "10." | "prune mempool" => {
                // confirm with user that they want to clear the mempool
                println!("This will clear your current mempool, you will not be able to confirm/view any of the previously broadcasted to you");
                println!("Do you want to clear your mempool? (yes/no)");
                let prune_option = read_string();
                println!();

                // prune the mempool if the user entered yes
                match prune_option.as_str() {
                    "yes" => controller.blockchain_prune_mempool(),
                    _ => continue
                }
                
                println!("Mempool pruned");
                println!();
            },
            "11" | "11." | "back" => {
                break;
            },
            _ => {}
        }
    }
}

fn perform_transaction_options(controller: &mut Controller) {
    let transaction_options = vec!["Transaction Options:", "A -> B", "Validator enable", "Validator revoke", "Back"];

    loop {
        // present transaction options to user
        print_options(&transaction_options);

        // get user choice
        let option_input = read_string().to_lowercase();
        println!();

        match option_input.as_str() {
            "1" | "1." | "a -> b" | "a->b" | "ab" | "a b" | "a" | "b" => {
                loop {
                    // prompt the user for the recipient address
                    println!("Enter the recipient (case sensitive) or \"exit\":");
                    let address_string = read_string();
                    println!();

                    // exit if they entered exit
                    if address_string.to_lowercase() == "exit" {
                        break 
                    }

                    // attempt to convert the address string to a vector
                    let address_arr: [u8; BLOCK_ADDRESS_SIZE] = match address_string.as_bytes().try_into() {
                        Ok(address ) => address,
                        Err(_) => {
                            println!("Invalid address");
                            println!();
                            continue;
                        }
                    };

                    // address checksum checking
                    if !controller.check_address_checksum(address_arr) {
                        println!("Invalid address, please check for typos");
                        println!();
                        continue;
                    }

                    loop {
                        // prompt the user for the amount theyd like to send
                        let balance = controller.wallet_get_balance();
                        println!("Current balance: {:.8} BLO", balance);
                        println!("Enter the amount of BLO you'd like to send or \"exit\":");
                        let amount_string = read_string();
                        println!();

                        // exit if they entered exit
                        if amount_string.to_lowercase() == "exit" {
                            break;
                        }
                        
                        // parse the amount
                        let amount = match amount_string.parse::<f64>() {
                            Ok(amount) => amount,
                            Err(_) => continue
                        };

                        // confirm the user has entered a non-negative number
                        if amount <= 0.0 {
                            println!("Enter an amount larger than 0");
                            println!();
                            continue;
                        }

                        // ensure the user has entered a prcision of 8 deicmal places or less
                        let amount_string_parts = amount_string.split('.').collect::<Vec<&str>>();
                        if amount_string_parts.len() == 2 {
                            if amount_string_parts[1].len() > 8 {
                                println!("Please enter an amount of block with a maximum of 8 decmal places (0.00000001 = 1 bit)");
                                println!();
                                continue;
                            }
                        }

                        // confirm the user has sufficient funds 
                        if amount > balance {
                            println!("Insufficient funds");
                            println!();
                            continue;
                        }
                        
                        loop {
                            // prompt the user for the fee theyd like to use
                            println!("Enter the fee you'd like to attach to your transaction (in BLO) or \"exit\":");
                            let fee_string = read_string();
                            println!();
                            
                            // exit if they entered exit
                            if fee_string.to_lowercase() == "exit" {
                                break;
                            }
                            
                            // parse the fee
                            let fee = match fee_string.parse::<f64>() {
                                Ok(fee) => fee,
                                Err(_) => continue
                            };

                            // confirm the user has entered a positive number
                            if fee < 0.0 {
                                println!("Enter a positive number for the fee");
                                println!();
                                continue;
                            }

                            // ensure the user has entered a prcision of 8 deicmal places or less
                            let fee_string_parts = fee_string.split('.').collect::<Vec<&str>>();
                            if fee_string_parts.len() == 2 {
                                if fee_string_parts[1].len() > 8 {
                                    println!("Please enter a fee with a maximum of 8 decmal places (0.00000001 = 1 bit)");
                                    println!();
                                    continue;
                                }
                            }

                            // confirm the user has sufficient funds
                            if fee > balance - amount {
                                println!("Insufficient funds");
                                println!();
                                continue;
                            }

                            // create and broadcast the transaction
                            if controller.transaction_create_a_b(address_arr, (amount * *LOWEST_DENOMINATION_PER_COIN).ceil() as u64, (fee * *LOWEST_DENOMINATION_PER_COIN).ceil() as u64) {
                                println!("Successfully broadcast transaction");
                                println!();
                            } else {
                                println!("Failed broadcasting transaction");
                                println!();
                            }
                            break;
                        }
                        break;
                    }
                    break;
                }
            },
            "2" | "2." | "validator enable" | "enable" => {
                loop {
                    // prompt the user for the amount theyd like to stake
                    let balance = controller.wallet_get_balance();
                    println!("Current balance: {:.8} BLO", balance);
                    println!("Enter the amount of BLO you'd like to stake or \"exit\":");
                    let amount_string = read_string();
                    println!();

                    // exit if they entered exit
                    if amount_string.to_lowercase() == "exit" {
                        break;
                    }
                    
                    // parse the amount
                    let amount = match amount_string.parse::<f64>() {
                        Ok(amount) => amount,
                        Err(_) => continue
                    };

                    // confirm the user has entered a non-negative number
                    if amount < 0.0 {
                        println!("Enter a positive number for the amount");
                        println!();
                        continue;
                    }

                    // ensure the user has entered a prcision of 8 deicmal places or less
                    let amount_string_parts = amount_string.split('.').collect::<Vec<&str>>();
                    if amount_string_parts.len() == 2 {
                        if amount_string_parts[1].len() > 8 {
                            println!("Please enter an amount of BLO with a maximum of 8 decmal places (0.00000001 = 1 bit)");
                            println!();
                            continue;
                        }
                    }

                    // confirm the user has sufficient funds
                    if amount > balance {
                        println!("Insufficient funds");
                        println!();
                        continue;
                    }
                    
                    loop {
                        // prompt the user for the fee theyd like to use
                        println!("Enter the fee you'd like to attach to your transaction (in BLO) or \"exit\":");
                        let fee_string = read_string();
                        println!();
                        
                        // exit if they entered exit
                        if fee_string.to_lowercase() == "exit" {
                            break;
                        }
                        
                        // parse the fee
                        let fee = match fee_string.parse::<f64>() {
                            Ok(fee) => fee,
                            Err(_) => continue
                        };

                        // confirm the user has entered a non-negative number
                        if fee < 0.0 {
                            println!("Enter a positive number for the fee");
                            println!();
                            continue;
                        }

                        // ensure the user has entered a prcision of 8 deicmal places or less
                        let fee_string_parts = fee_string.split('.').collect::<Vec<&str>>();
                        if fee_string_parts.len() == 2 {
                            if fee_string_parts[1].len() > 8 {
                                println!("Please enter a fee with a maximum of 8 decmal places (0.00000001 = 1 bit)");
                                println!();
                                continue;
                            }
                        }

                        // confirm the user has sufficient funds
                        if fee > balance - amount {
                            println!("Insufficient funds");
                            println!();
                            continue;
                        }

                        // create and broadcast the transaction
                        if controller.transaction_create_validator_enable((amount * *LOWEST_DENOMINATION_PER_COIN).ceil() as u64, (fee * *LOWEST_DENOMINATION_PER_COIN).ceil() as u64) {
                            println!("Successfully broadcast transaction");
                            println!();
                        } else {
                            println!("Failed broadcasting transaction");
                            println!();
                        }
                        break;
                    }
                    break;
                }
            },
            "3" | "3." | "validator revoke" | "revoke" => {
                // get the amount
                loop {
                    // check if the account is currently staking
                    let account = match controller.blockchain_get_account(&controller.wallet_get_address()) {
                        Some(account) => {
                            if !account.get_validator() {
                                println!("You are not currently a validator on the blockchain");
                                break;
                            }
                            account
                        }
                        None => {
                            println!("You are not currently a validator on the blockchain");
                            break;
                        }
                    };

                    println!("This will revoke your entire stake of: {:.8} BLO", account.get_stake());

                    // prompt the user for the fee theyd like to use
                    let balance = controller.wallet_get_balance();
                    println!("Current balance: {:.8} BLO", balance);
                    println!("Enter the fee you'd like to attach to your transaction (in BLO) or \"exit\":");
                    let fee_string = read_string();
                    println!();

                    // exit if they entered exit
                    if fee_string.to_lowercase() == "exit" {
                        break;
                    }
                    
                    // parse the fee
                    let fee = match fee_string.parse::<f64>() {
                        Ok(fee) => fee,
                        Err(_) => continue
                    };

                    // confirm the user has entered a non-negative number
                    if fee < 0.0 {
                        println!("Enter a positive number for the fee");
                        println!();
                        continue;
                    }

                    // ensure the user has entered a prcision of 8 deicmal places or less
                    let fee_string_parts = fee_string.split('.').collect::<Vec<&str>>();
                    if fee_string_parts.len() == 2 {
                        if fee_string_parts[1].len() > 8 {
                            println!("Please enter a fee with a maximum of 8 decmal places (0.00000001 = 1 bit)");
                            println!();
                            continue;
                        }
                    }

                    // confirm the user has sufficient funds
                    if fee > balance {
                        println!("Insufficient funds");
                        println!();
                        continue;
                    }

                    // create and broadcast the transaction
                    if controller.transaction_create_validator_revoke(account.get_stake(), (fee * *LOWEST_DENOMINATION_PER_COIN).ceil() as u64) {
                        println!("Successfully broadcast transaction");
                        println!();
                    } else {
                        println!("Failed broadcasting transaction");
                        println!();
                    }
                    break;
                }
            },
            "4" | "4." | "back" => {
                break;
            },
            _ => {}
        }
    }
}

fn perform_about_options(controller: &Controller) {
    let about_options = vec!["About options", "Node version", "Wallet config values", "Validator config values", "Back"];

    loop {
        // present transaction options to user
        print_options(&about_options);

        // get user choice
        let option_input = read_string().to_lowercase();
        println!();

        match option_input.as_str() {
            "1" | "1." | "node version" | "node" | "version" => {
                println!("Node version: {}", NODE_VERSION);
                println!();
            },
            "2" | "2." | "wallet config values" | "wallet config" | "wallet" => {
                println!("Wallet config values:");
                println!("{:?}", controller.about_wallet_config());
                println!();
            },
            "3" | "3." | "validator config values" | "validator config" | "validator" => {
                println!("Validator config values:");
                println!("{:?}", controller.about_validator_config());
                println!();
            }
            "4" | "4." | "back" => {
                break;
            },
            _ => {}
        }
    }
}

fn print_options(options: &Vec<&str>) {
    // print an options array to the user
    for (index, option) in options.iter().enumerate() {
        // the first item in the array will be a "header" of sorts for the options list ex. "Transaction Options:"
        if index == 0 {
            println!("{}", option)
        } else {
            println!("  {}: {}", index, option);
        }
    }
    println!();

    // prompt the user to enter an option number
    print!("Enter an option number: ");
    match io::stdout().flush() {
        Ok(_) => (),
        Err(_) => ()
    };
}

fn read_string() -> String {
    // helper function to read user input from the command line, removes leading and trailing spaces
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => (),
        Err(_) => return String::new()
    };

    input.trim().to_string()
}
