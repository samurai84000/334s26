use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256, Hashable};
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub parent: H256,
    pub nonce: u32,
    pub difficulty: H256,
    pub timestamp: u128,
    pub merkle_root: H256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    pub transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: Header,
    pub content: Content,
}

impl Hashable for Header {
    fn hash(&self) -> H256 {
        let bytes = bincode::serialize(&self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &bytes).into()
    }
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        self.header.hash()
    }
}

impl Block {
    pub fn genesis() -> Block {
        let header = Header {
            parent: Default::default(),
            nonce: 0,
            difficulty: [0u8; 32].into(), // You can use default_difficulty() here
            timestamp: 0,
            merkle_root: Default::default(),
        };
        Block { header, content: Content { transactions: vec![] } }
    }
}

pub fn default_difficulty() -> [u8; 32] {
    let mut difficulty = [0u8; 32];
    difficulty[0] = 1; 
    difficulty
}

#[cfg(any(test, test_utilities))]
pub mod test {
    use super::*;
    use crate::crypto::merkle::MerkleTree;

    pub fn generate_random_block(parent: &H256) -> Block {
        let transactions: Vec<Transaction> = vec![Default::default()];
        let merkle_tree = MerkleTree::new(&transactions);
        
        let header = Header {
            parent: *parent,
            nonce: rand::random::<u32>(),
            difficulty: default_difficulty().into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            merkle_root: merkle_tree.root(),
        };
        
        Block {
            header,
            content: Content { transactions },
        }
    }
}