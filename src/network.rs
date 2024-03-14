use std::{net::Ipv4Addr, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::config::NetworkConfig;
use crate::constants::NODE_VERSION;
use crate::transaction::Transaction;

use protoping::ping_service_client::PingServiceClient;
use protoping::BroadcastPingRequest;

use prototransaction::transaction_service_client::TransactionServiceClient;
use prototransaction::BroadcastTransactionRequest;

use protoblock::block_service_client::BlockServiceClient;
use protoblock::BroadcastBlockRequest;

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
            let mut client = match PingServiceClient::connect(format!("http://{}:{}", peer.ip, peer.port)).await {
                Ok(client) => client,
                Err(_) => {
                    println!("Unable to connect to peer: {}:{} during initial connect, removing from peer list. This will not remove this peer from your config file.", peer.ip, peer.port);
                    continue
                }
            };

            // create the request
            let request = tonic::Request::new(BroadcastPingRequest {
                version: String::from(NODE_VERSION),
            });
            
            // make the request to the peer and get a response
            let response = match client.broadcast_ping(request).await {
                Ok(response) => response.into_inner(),
                Err(_) => {
                    println!("Unable to ping peer: {}:{} during initial connect, removing from peer list. This will not remove this peer from your config file.", peer.ip, peer.port);
                    continue
                }
            };

            // ToDo: need an api version vs node version

            // parse ping response

            // semantic version regex obtained from: https://semver.org/#is-there-a-suggested-regular-expression-regex-to-check-a-semver-string
            // edited to only allow for major.minor.patch, does not allow pre-release, or build-metadata to be present in the version number 
            let version_regex = Regex::new(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$").unwrap();
            // confirm the version recevied is in proper semver format: https://semver.org
            if !version_regex.is_match(&response.version) {
                println!("Invalid version ({}) received from peer: {}:{}, removing from peer list. This will not remove this peer from your config file.", response.version, peer.ip, peer.port);
                continue
            }
            // confirm the version of the peer that is received is compatible with the current running node software
            if NODE_VERSION.split(".").collect::<Vec<&str>>()[0] != response.version.split(".").collect::<Vec<&str>>()[0] {
                println!("Incompatible version ({}) received from peer: {}:{}, removing from peer list. This will not remove this peer from your config file.", response.version, peer.ip, peer.port);
                continue
            }

            peer.valid = true;
        }

        // remove any peers that were unable to connect
        self.peer_list.retain(|peer| {
            peer.valid
        });
    }

    pub async fn broadcast_transaction(&mut self, transaction: &Transaction) -> Option<Vec<Peer>> {
        let mut successful_broadcasts = vec![];

        // only attempt to broadcast transactions if not running a local blockchain
        if !self.get_local_blockchain() {
            for peer in self.peer_list.iter_mut() { 
                // attempt to establish connection with the peer
                let mut client = match TransactionServiceClient::connect(format!("http://{}:{}", peer.ip, peer.port)).await {
                    Ok(client) => client,
                    Err(_) => {
                        println!("Unable to connect to peer: {}:{} ", peer.ip, peer.port);
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
                        continue
                    }
                };

                // parse transaction broadcast response
                if !response.ok {
                    println!("Broadcast transaction rejection from peer: {}:{}", peer.ip, peer.port);
                    continue
                } 

                successful_broadcasts.push(peer.to_owned());
            }
        }

        Some(successful_broadcasts)
    }

    pub async fn broadcast_block(&mut self, block: &Block) -> Option<Vec<Peer>> {
        let mut successful_broadcasts = vec![];

        // only attempt to broadcast transactions if not running a local blockchain
        if !self.get_local_blockchain() {
            for peer in self.peer_list.iter_mut() { 
                // attempt to establish connection with the peer
                let mut client = match BlockServiceClient::connect(format!("http://{}:{}", peer.ip, peer.port)).await {
                    Ok(client) => client,
                    Err(_) => {
                        println!("Unable to connect to peer: {}:{} ", peer.ip, peer.port);
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
                        continue
                    }
                };

                // parse transaction broadcast response
                if !response.ok {
                    println!("Broadcast block rejection from peer: {}:{}", peer.ip, peer.port);
                    continue
                } 

                successful_broadcasts.push(peer.to_owned());
            }
        }

        Some(successful_broadcasts)
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
    fn new (socket_address: &str) -> Option<Peer> {
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
}