use std::{net::Ipv4Addr, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::config::NetworkConfig;
use crate::constants::NODE_VERSION;
use crate::transaction::Transaction;

use ping::ping_client::PingClient;
use ping::PingRequest;

pub mod ping {
    tonic::include_proto!("ping");
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

    #[tokio::main]
    pub async fn initial_connect(&mut self) {
        // ping each peer in the peer list and mark them as invalid if unable to connect, or received an invalid ping response
        for peer in self.peer_list.iter_mut() {
            // attempt to establish connection with the peer
            let mut client = match PingClient::connect(format!("http://{}:{}", peer.ip, peer.port)).await {
                Ok(client) => client,
                Err(_) => {
                    println!("Unable to connect to peer: {}:{} during initial connect, removing from peer list. This will not remove this peer from your config file.", peer.ip, peer.port);
                    continue
                }
            };

            // create the request
            let request = tonic::Request::new(PingRequest {
                version: String::from(NODE_VERSION),
            });
            
            // make the request to the peer and get a response
            let response = match client.ping(request).await {
                Ok(response) => response.into_inner(),
                Err(_) => {
                    println!("Unable to ping peer: {}:{} during initial connect, removing from peer list. This will not remove this peer from your config file.", peer.ip, peer.port);
                    continue
                }
            };

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

    pub fn broadcast_transaction(&self, transaction: &Transaction) -> bool {
        // ToDo: 
        let serialized_transaction = transaction.serialize_tx();
        true
    }

    pub fn broadcast_block(&self, block: &Block) -> bool {
        // ToDo: 
        let serialized_block = block.serialize_block();
        true
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