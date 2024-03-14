use std::mem;

use bincode::{Options, ErrorKind};
use k256::ecdsa::Signature;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

use crate::transaction::Transaction;

// ToDo: May need to remove transaction_count variable, not because its uneeded, but because during serialization serde already adds a transaction count before the transaction vector
// This duplicates the transaction count in the serialized structure. Probably should have a customized serialization function, so its not just implicity there
// but thats extra work thats probably not needed at this time

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    block_size: u32,
    block_header: BlockHeader,
    //transaction_count: u32,
    transactions: Vec<Transaction>,
    signature: Signature,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockHeader {
    version: u32,
    prev_hash: [u8; 32],
    merkle_root: [u8; 32],
    timestamp: u64,
}

#[derive(Debug)]
struct MerkleTree {
    leaves: Vec<MerkleNode>
}

#[derive(Debug)]
struct MerkleNode {
    parent: Option<Box<MerkleNode>>,
    hash: Vec<u8>
}

impl Block {
    pub fn new(version: u32, prev_hash: [u8; 32], timestamp: u64, transactions: &Vec<Transaction>, signature: Signature) -> Self {
        // calculate the merkle root
        let merkle_root:[u8; 32] = BlockHeader::calculate_merkle_root(transactions.clone()).try_into().unwrap();
        // create the block header
        let block_header = BlockHeader {
            version,
            prev_hash,
            merkle_root,
            timestamp,
        };

        // get the transaction count
        //let transaction_count: u32 = transactions.len().try_into().unwrap();

        // get the size of the block (besides the block_size field itself)
        let block_size = mem::size_of::<BlockHeader>() /*+ mem::size_of_val(&transaction_count)*/ + (mem::size_of::<Transaction>() * transactions.len() + mem::size_of::<Signature>());

        Block {
            block_size: block_size.try_into().unwrap(),
            block_header,
            //transaction_count,
            transactions: transactions.clone(),
            signature,
        }

    }

    pub fn prev_hash(&self) -> [u8; 32] {
        self.block_header.prev_hash
    }

    pub fn merkle_root(&self) -> [u8; 32] {
        self.block_header.merkle_root
    }

    pub fn serialize_block(&self) -> Vec<u8> {
        bincode::DefaultOptions::new()
            .allow_trailing_bytes()
            .with_fixint_encoding()
            .with_big_endian()
            .serialize(self).unwrap()
    }

    pub fn from(raw: Vec<u8>) -> Result<Self, Box<ErrorKind>> {
        bincode::DefaultOptions::new()
            .allow_trailing_bytes()
            .with_fixint_encoding()
            .with_big_endian()
            .deserialize(&raw)
    }

    pub fn get_transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn serialize_hash_block_header(&self) -> Vec<u8> {
        self.block_header.serialize_hash_block_header()
    }

    pub fn get_timesamp(&self) -> u64 {
        self.block_header.timestamp
    }

    pub fn get_signature(&self) -> Signature {
        self.signature
    }

    pub fn get_block_size(&self) -> u32 {
        self.block_size
    }

    pub fn get_version(&self) -> u32 {
        self.block_header.version
    }
}

impl BlockHeader {
    pub fn new(version: u32, prev_hash: [u8; 32], merkle_root: [u8; 32], timestamp: u64) -> Self {
        Self {
            version,
            prev_hash,
            merkle_root,
            timestamp
        }
    }
    // ToDo: This function doesn't ever create a full merkle tree, it creates each level of a merkle tree sequentially and returns just the final merkle root
    // To simplify transaction validation for light nodes (which don't and probably won't ever exist on block) a true merkle tree would be needed so that a merkle path can be used to validate single transactions
    pub fn calculate_merkle_root(transactions: Vec<Transaction>) -> Vec<u8> {
        let mut merkle_nodes: Vec<MerkleNode> = vec![];

        for transaction in transactions {
            // create merkle nodes from transactions with None as parent (for now)
            let tx_hash = transaction.serialize_hash_tx();
            merkle_nodes.push(
                MerkleNode {
                    parent: None,
                    hash: tx_hash
                }
            )
        }

        // create a merkle tree with the leaves that were just created
        let mut merkle_tree = MerkleTree {
            leaves: merkle_nodes
        };

        // combine all merkle nodes (leaves) up the tree until there is jsut a merkle root
        while merkle_tree.leaves.len() > 1 {
            // vector to hold the new leaves that are products of the previous leaves combined with their neighboring leaf
            let mut new_leaves: Vec<MerkleNode> = vec![];

            for (index, leaf) in merkle_tree.leaves.iter().enumerate() {
                // skip every other node since nodes even number nodes are combined with the odd number node to its "right"
                if index % 2 != 0 {
                    continue;
                }

                // get the hash of the first leaf
                let mut hash1 = leaf.hash.clone();

                // attempt to get the hash of the next node, BUT if there is no next node, because there an odd number of nodes at this level, then duplicate the previous nodes hash
                let mut hash2: Vec<u8> = match merkle_tree.leaves.get(index + 1) {
                    Some(node) => node.hash.clone(),
                    None => hash1.clone()
                };

                // combine the 2 hashes together
                hash1.append(&mut hash2);

                // hash the combined hashes, sha256(current_leaf_hash | next_leaf_hash)
                let mut sha256_hasher: Sha256 = Sha256::new();
                sha256_hasher.update(hash1);
                let combined_hash = sha256_hasher.finalize().to_vec();

                new_leaves.push(
                    MerkleNode {
                        parent: None,
                        hash: combined_hash
                    }
                );
            }

            // set the merkle tree to one with just the new combined leaves. This new merkle tree will have exactly (if even) half of the nodes as the previous merkle tree
            /*
            
                    []
                  /   \
                []    []   <- new tree height (2 total nodes)
               / \   / \
              [] [] [] []  <- old tree height (4 total nodes)

            */
            merkle_tree = MerkleTree {
                leaves: new_leaves
            }
        }
        
        merkle_tree.leaves[0].hash.clone()
    }

    pub fn serialize_block_header(&self) -> Vec<u8> {
        bincode::DefaultOptions::new()
            .allow_trailing_bytes()
            .with_fixint_encoding()
            .with_big_endian()
            .serialize(self).unwrap()
    }

    pub fn serialize_hash_block_header(&self) -> Vec<u8> {
        // serialize block header
        let serialized_block_header = self.serialize_block_header();
        // sha256(serialized block header)
        Self::hash_serialized_block_header(serialized_block_header)
    }

    pub fn from(raw: Vec<u8>) -> Result<Self, Box<ErrorKind>> {
        bincode::DefaultOptions::new()
            .allow_trailing_bytes()
            .with_fixint_encoding()
            .with_big_endian()
            .deserialize(&raw)
    }

    fn hash_serialized_block_header(serialized_block_header: Vec<u8>) -> Vec<u8> {
        // sha256(serialized block header)
        let mut sha256_hasher: Sha256 = Sha256::new();
        sha256_hasher.update(serialized_block_header);
        sha256_hasher.finalize().to_vec()
    }
}