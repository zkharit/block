use std::{net::Ipv4Addr, str::FromStr};

use k256::ecdsa::Signature;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::block::{Block, BlockHeader};
use crate::config::NetworkConfig;
use crate::constants::{API_VERSION, BLOCK_ADDRESS_SIZE, COMPRESSED_PUBLIC_KEY_SIZE, NODE_VERSION};
use crate::transaction::Transaction;

use protoping::ping_service_client::PingServiceClient;
use protoping::BroadcastPingRequest;

use prototransaction::transaction_service_client::TransactionServiceClient;
use prototransaction::BroadcastTransactionRequest;

use protoblock::block_service_client::BlockServiceClient;
use protoblock::{BroadcastBlockRequest, GetBlockRequest, GetBlockHeightRequest};

pub mod protoping {
    tonic::include_proto!("block.ping");
}

pub mod prototransaction {
    tonic::include_proto!("block.transaction");
}

pub mod protoblock {
    tonic::include_proto!("block.block");
}

pub struct Network {
    // config for the network
    config: NetworkConfig,
    // peer list
    peer_list: Vec<Peer>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Peer {
    ip: Ipv4Addr,
    port: u16,
    valid: bool
}

impl Network {
    pub fn new(config: NetworkConfig) -> Self {
        let mut peer_list = vec![];

        // attempt to build a peer object from each of the ip/port combos passed in peer_list field of the network config
        // skip it if its an invalid ipv4:port format
        for socket_address in config.get_peer_list() {
            match Peer::new(&socket_address) {
                Some(peer) => peer_list.push(peer),
                None => continue
            }
        }

        Self {
            config,
            peer_list,
        }
    }

    pub async fn initial_connect(&mut self) {
        // ping each peer in the peer list and mark them as invalid if unable to connect, or received an invalid ping response
        for peer in self.peer_list.iter_mut() {
            // attempt to establish connection with the peer
            // ToDo: can change timeout through tonic::transport::channel https://docs.rs/tonic/latest/tonic/transport/index.html#client
            let mut client = match PingServiceClient::connect(format!("http://{}:{}", peer.ip, peer.port)).await {
                Ok(client) => client,
                Err(_) => {
                    println!("Unable to connect to peer: {}:{} during initial connect, removing from peer list. This will not remove this peer from your config file.", peer.ip, peer.port);
                    println!();
                    continue
                }
            };

            // create the request
            let request = tonic::Request::new(BroadcastPingRequest {
                node_version: String::from(NODE_VERSION),
                api_version: String::from(API_VERSION)
            });
            
            // make the request to the peer and get a response
            let response = match client.broadcast_ping(request).await {
                Ok(response) => response.into_inner(),
                Err(_) => {
                    println!("Unable to ping peer: {}:{} during initial connect, removing from peer list. This will not remove this peer from your config file.", peer.ip, peer.port);
                    println!();
                    continue
                }
            };

            // parse ping response

            // semantic version regex obtained from: https://semver.org/#is-there-a-suggested-regular-expression-regex-to-check-a-semver-string
            // edited to only allow for major.minor.patch, does not allow pre-release, or build-metadata to be present in the version number 
            let version_regex = Regex::new(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$").unwrap();
            // confirm the node version recevied is in proper semver format: https://semver.org
            if !version_regex.is_match(&response.node_version) {
                println!("Invalid node version ({}) received from peer: {}:{}, removing from peer list. This will not remove this peer from your config file.", response.node_version, peer.ip, peer.port);
                println!();
                continue
            }
            // confirm the version of the peer that is received is compatible with the current running node software
            if NODE_VERSION.split(".").collect::<Vec<&str>>()[0] != response.node_version.split(".").collect::<Vec<&str>>()[0] {
                println!("Incompatible node version ({}) received from peer: {}:{}, removing from peer list. This will not remove this peer from your config file.", response.node_version, peer.ip, peer.port);
                println!();
                continue
            }

            // confirm the api version recevied is in proper semver format: https://semver.org
            if !version_regex.is_match(&response.api_version) {
                println!("Invalid api version ({}) received from peer: {}:{}, removing from peer list. This will not remove this peer from your config file.", response.api_version, peer.ip, peer.port);
                println!();
                continue
            }
            // confirm the api version of the peer that is received is compatible with the current running node software
            if API_VERSION.split(".").collect::<Vec<&str>>()[0] != response.api_version.split(".").collect::<Vec<&str>>()[0] {
                println!("Incompatible api version ({}) received from peer: {}:{}, removing from peer list. This will not remove this peer from your config file.", response.api_version, peer.ip, peer.port);
                println!();
                continue
            }

            peer.valid = true;
        }

        // remove any peers that were unable to connect
        self.peer_list.retain(|peer| {
            peer.valid
        });
    }

    pub async fn ping_peer(&mut self, peer: &mut Peer) -> bool {
         // attempt to establish connection with the peer
         let mut client = match PingServiceClient::connect(format!("http://{}:{}", peer.ip, peer.port)).await {
            Ok(client) => client,
            Err(_) => {
                println!("Unable to connect to peer: {}:{}", peer.ip, peer.port);
                println!();
                return false
            }
        };

        // create the request
        let request = tonic::Request::new(BroadcastPingRequest {
            node_version: String::from(NODE_VERSION),
            api_version: String::from(API_VERSION),
        });
        
        // make the request to the peer and get a response
        let response = match client.broadcast_ping(request).await {
            Ok(response) => response.into_inner(),
            Err(_) => {
                println!("Unable to ping peer: {}:{}", peer.ip, peer.port);
                println!();
                return false
            }
        };

        // parse ping response

        // semantic version regex obtained from: https://semver.org/#is-there-a-suggested-regular-expression-regex-to-check-a-semver-string
        // edited to only allow for major.minor.patch, does not allow pre-release, or build-metadata to be present in the version number 
        let version_regex = Regex::new(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$").unwrap();
        // confirm the node version recevied is in proper semver format: https://semver.org
        if !version_regex.is_match(&response.node_version) {
            println!("Invalid node version ({}) received from peer: {}:{}", response.node_version, peer.ip, peer.port);
            println!();
            return false
        }
        // confirm the node version of the peer that is received is compatible with the current running node software
        if NODE_VERSION.split(".").collect::<Vec<&str>>()[0] != response.node_version.split(".").collect::<Vec<&str>>()[0] {
            println!("Incompatible node version ({}) received from peer: {}:{}", response.node_version, peer.ip, peer.port);
            println!();
            return false
        }

        // confirm the api version recevied is in proper semver format: https://semver.org
        if !version_regex.is_match(&response.api_version) {
            println!("Invalid api version ({}) received from peer: {}:{}", response.api_version, peer.ip, peer.port);
            println!();
            return false
        }
        // confirm the api version of the peer that is received is compatible with the current running node software
        if API_VERSION.split(".").collect::<Vec<&str>>()[0] != response.api_version.split(".").collect::<Vec<&str>>()[0] {
            println!("Incompatible api version ({}) received from peer: {}:{}", response.api_version, peer.ip, peer.port);
            println!();
            return false
        }

        //set peer as valid if ping is successful
        peer.valid = true;

        true
    }

    pub async fn broadcast_transaction(&mut self, transaction: &Transaction) -> Option<Vec<Peer>> {
        let mut successful_broadcasts = vec![];

        // only attempt to broadcast transaction if not running a local blockchain
        if !self.get_local_blockchain() {
            for peer in self.peer_list.iter_mut() { 
                // attempt to establish connection with the peer
                let mut client = match TransactionServiceClient::connect(format!("http://{}:{}", peer.ip, peer.port)).await {
                    Ok(client) => client,
                    Err(_) => {
                        println!("Unable to connect to peer: {}:{} ", peer.ip, peer.port);
                        println!();
                        continue
                    }
                };

                // create the request
                let request = tonic::Request::new(BroadcastTransactionRequest {
                    transaction: Some(prototransaction::Transaction {
                        version: transaction.version.into(),
                        amount: transaction.amount,
                        fee: transaction.fee,
                        recipient: transaction.recipient.to_vec(),
                        sender: transaction.sender.to_vec(),
                        signature: transaction.signature.to_vec(),
                        nonce: transaction.nonce,
                    })
                });

                // make the request to the peer and get a response
                let response = match client.broadcast_transaction(request).await {
                    Ok(response) => response.into_inner(),
                    Err(_) => {
                        println!("Unable to broadcast transaction to peer: {}:{}", peer.ip, peer.port);
                        println!();
                        continue
                    }
                };

                // parse transaction broadcast response
                if !response.ok {
                    println!("Broadcast transaction rejection from peer: {}:{}", peer.ip, peer.port);
                    println!();
                    continue
                } 

                successful_broadcasts.push(peer.to_owned());
            }
        }

        Some(successful_broadcasts)
    }

    pub async fn broadcast_block(&mut self, block: &Block) -> Option<Vec<Peer>> {
        let mut successful_broadcasts = vec![];

        // only attempt to broadcast block if not running a local blockchain
        if !self.get_local_blockchain() {
            for peer in self.peer_list.iter_mut() { 
                // attempt to establish connection with the peer
                let mut client = match BlockServiceClient::connect(format!("http://{}:{}", peer.ip, peer.port)).await {
                    Ok(client) => client,
                    Err(_) => {
                        println!("Unable to connect to peer: {}:{} ", peer.ip, peer.port);
                        println!();
                        continue
                    }
                };

                // create the request
                let request = tonic::Request::new(BroadcastBlockRequest {
                    block: Some(protoblock::Block {
                        block_size: block.get_block_size(),
                        block_header: Some(protoblock::BlockHeader {
                            version: block.get_version(),
                            prev_hash: block.prev_hash().to_vec(),
                            merkle_root: block.merkle_root().to_vec(),
                            timestamp: block.get_timesamp(),
                        }),
                        transactions: block.get_transactions().iter().map(|transaction| protoblock::Transaction {
                            version: transaction.version.into(),
                            amount: transaction.amount,
                            fee: transaction.fee,
                            recipient: transaction.recipient.to_vec(),
                            sender: transaction.sender.to_vec(),
                            signature: transaction.signature.to_vec(),
                            nonce: transaction.nonce,
                        }).collect(),
                        signature: block.get_signature().to_vec(),
                    })
                });

                // make the request to the peer and get a response
                let response = match client.broadcast_block(request).await {
                    Ok(response) => response.into_inner(),
                    Err(_) => {
                        println!("Unable to broadcast block to peer: {}:{}", peer.ip, peer.port);
                        println!();
                        continue
                    }
                };

                // parse transaction broadcast response
                if !response.ok {
                    println!("Broadcast block rejection from peer: {}:{}", peer.ip, peer.port);
                    println!();
                    continue
                } 

                successful_broadcasts.push(peer.to_owned());
            }
        }

        Some(successful_broadcasts)
    }

    pub async fn get_block_height(&mut self, peer: &Peer) -> Option<u64> {
        // only attempt to get block height if not running a local blockchain
        if !self.get_local_blockchain() {
            // attempt to establish connection with the peer
            let mut client = match BlockServiceClient::connect(format!("http://{}:{}", peer.ip, peer.port)).await {
                Ok(client) => client,
                Err(_) => {
                    println!("Unable to connect to peer: {}:{} ", peer.ip, peer.port);
                    println!();
                    return None
                }
            };

            // create the request
            let request = tonic::Request::new(GetBlockHeightRequest {});

            // make the request to the peer and get a response
            let response = match client.get_block_height(request).await {
                Ok(response) => response.into_inner(),
                Err(_) => {
                    println!("Unable to get block height from peer: {}:{}", peer.ip, peer.port);
                    println!();
                    return None
                }
            };

            Some(response.block_height)
        } else {
            return None
        }
    }

    pub async fn get_block(&mut self, peer: &Peer, block_height: u64) -> Option<Block> {
        // only attempt to broadcast transactions if not running a local blockchain
        if !self.get_local_blockchain() {
            // attempt to establish connection with the peer
            let mut client = match BlockServiceClient::connect(format!("http://{}:{}", peer.ip, peer.port)).await {
                Ok(client) => client,
                Err(_) => {
                    println!("Unable to connect to peer: {}:{} ", peer.ip, peer.port);
                    println!();
                    return None
                }
            };

            // create the request
            let request = tonic::Request::new(GetBlockRequest {
                block_height
            });

            // make the request to the peer and get a response
            let response = match client.get_block(request).await {
                Ok(response) => response.into_inner(),
                Err(_) => {
                    println!("Unable to obtain block at height {} from peer: {}:{}", block_height, peer.ip, peer.port);
                    println!();
                    return None
                }
            };

            // obtain the block from the response
            let protoblock = match response.block {
                Some(protoblock) => protoblock,
                None => {
                    println!("Peer {}:{} doesnt have a block for block height {}", peer.ip, peer.port, block_height);
                    println!();
                    return None
                }
            };

            // convert the protoblock::block to a block::block, there are some type mismatches here that need to be considered

            // obtain the block size
            let block_size = protoblock.block_size;
            // obtain the signature
            let signature = match Signature::from_slice(&protoblock.signature) {
                Ok(signature) => signature,
                Err(_) => {
                    println!("Improperly formatted signature from peer {}:{} in block at height {}", peer.ip, peer.port, block_height);
                    println!();
                    return None
                }
            };
            // obtain the protoblock::blockheader
            let proto_block_header = match protoblock.block_header {
                Some(proto_block_header) => proto_block_header,
                None => {
                    println!("Failed obtaining block header from peer {}:{} at block height {}", peer.ip, peer.port, block_height);
                    println!();
                    return None
                }
            };
            // obtain the previous hash from the protoblock::blockheader
            let prev_hash: [u8; 32] = match proto_block_header.prev_hash.try_into() {
                Ok(prev_hash) => prev_hash,
                Err(_) => {
                    println!("Failed obtaining previous hash from peer {}:{} at block height {}", peer.ip, peer.port, block_height);
                    println!();
                    return None
                },
            };
            // obtain the merkle root from the protoblock::blockheader
            let merkle_root: [u8; 32] = match proto_block_header.merkle_root.try_into() {
                Ok(merkle_root) => merkle_root,
                Err(_) => {
                    println!("Failed obtaining merkle root from peer {}:{} at block height {}", peer.ip, peer.port, block_height);
                    println!();
                    return None
                },
            };
            // build the block::blockheader
            let block_header = BlockHeader::new(proto_block_header.version, prev_hash, merkle_root, proto_block_header.timestamp);
            // convert the protoblock::transactions to block::transactions need to cast u32 -> u8 here, need to be careful
            let mut transactions = vec![];
            for transaction in protoblock.transactions.iter() {
                // obtain the transaction version
                let version = match u8::try_from(transaction.version) {
                    Ok(version) => version,
                    Err(_) => {
                        println!("Failed obtaining version in transaction from peer {}:{} at block height {}", peer.ip, peer.port, block_height);
                        println!();
                        return None
                    }
                };
                // obtain the amount
                let amount = transaction.amount;
                // obtain the fee
                let fee = transaction.fee;
                // obtain the recipient
                let recipient: [u8; BLOCK_ADDRESS_SIZE] = match transaction.recipient.clone().try_into() {
                    Ok(recipient) => recipient,
                    Err(_) => {
                        println!("Failed obtaining recipient in transaction from peer {}:{} at block height {}", peer.ip, peer.port, block_height);
                        println!();
                        return None
                    }
                };
                // obtain the sender
                let sender: [u8; COMPRESSED_PUBLIC_KEY_SIZE] = match transaction.sender.clone().try_into() {
                    Ok(sender) => sender,
                    Err(_) => {
                        println!("Failed obtaining sender in transaction from peer {}:{} at block height {}", peer.ip, peer.port, block_height);
                        println!();
                        return None
                    }
                };
                // obtain the transaction signature
                let tx_signature = match Signature::from_slice(&transaction.signature) {
                    Ok(tx_signature) => tx_signature,
                    Err(_) => {
                        println!("Improperly formatted signature in transaction from peer {}:{} at block height {}", peer.ip, peer.port, block_height);
                        println!();
                        return None
                    }
                };
                // obtain the nonce
                let nonce = transaction.nonce;

                // obtain the amount
                transactions.push(Transaction::new(version, amount, fee, recipient, sender, tx_signature, nonce));
            }

            return Some(Block::from_parts(block_size, block_header, transactions, signature))
        } else {
            return None
        }
    }

    pub fn get_local_blockchain(&self) -> bool {
        self.config.get_local_blockchain()
    }

    pub fn get_peer_list_len(&self) -> usize {
        self.peer_list.len()
    }

    pub fn get_peer_list(&self) -> Vec<Peer> {
        self.peer_list.clone()
    }
}

impl Peer {
    pub fn new (socket_address: &str) -> Option<Peer> {
        // split the ipv4:port string
        let address_parts = socket_address.split(":").collect::<Vec<&str>>();
        
        // if the string has more than 2 parts than it is invlaid
        if address_parts.len() != 2 {
            return None
        }

        // get the ipv4addr
        let ip = match Ipv4Addr::from_str(address_parts[0]) {
            Ok(ip) => ip,
            Err(_) => return None
        };

        // get the port number
        let port= match address_parts[1].parse::<u16>() {
            Ok(port) => port,
            Err(_) => return None
        };

        // set initial validity state of the peer as false, until connection has been tested
        let valid = false;

        Some (
            Self {
                ip,
                port,
                valid
        })
    }

    pub fn get_ip(&self) -> Ipv4Addr {
        self.ip
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }
}