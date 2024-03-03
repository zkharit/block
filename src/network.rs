use std::{net::Ipv4Addr, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::config::NetworkConfig;
use crate::transaction::Transaction;

pub struct Network {
    // config for the network
    config: NetworkConfig,
    // peer list
    peer_list: Vec<Peer>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Peer {
    ip: Ipv4Addr,
    port: u16
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

    pub fn initial_connect(&mut self) {
        for (index, peer) in self.peer_list.clone().iter_mut().enumerate() {
            // ToDo: do a ping, if they cant ping then remove them from the peer list
            // also maybe print out the removed peer
        }
    }

    pub fn broadcast_transaction(&self, transaction: &Transaction) -> bool {
        // ToDo: 
        true
    }

    pub fn get_local_blockchain(&self) -> bool {
        self.config.get_local_blockchain()
    }

    pub fn get_peer_list_len(&self) -> usize {
        self.peer_list.len()
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

        Some (
            Self {
                ip,
                port
        })
    }
}