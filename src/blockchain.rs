use crate::block::Block;
use crate::crypto::hash::{H256, Hashable};
use std::collections::HashMap;

pub struct Blockchain {
    hash_to_block: HashMap<H256, Block>,
    hash_to_height: HashMap<H256, u64>,
    tip: H256,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Block::genesis();
        let hash = genesis.hash();
        let mut hash_to_block = HashMap::new();
        let mut hash_to_height = HashMap::new();

        hash_to_block.insert(hash, genesis);
        hash_to_height.insert(hash, 0);

        Blockchain {
            hash_to_block,
            hash_to_height,
            tip: hash,
        }
    }

    pub fn insert(&mut self, block: &Block) {
        let hash = block.hash();
        let parent = block.header.parent;

        if let Some(&parent_height) = self.hash_to_height.get(&parent) {
            let height = parent_height + 1;
            self.hash_to_block.insert(hash, block.clone());
            self.hash_to_height.insert(hash, height);

            // Longest chain rule: Strictly greater height updates the tip
            let tip_height = *self.hash_to_height.get(&self.tip).unwrap();
            if height > tip_height {
                self.tip = hash;
            }
        }
    }

    pub fn tip(&self) -> H256 {
        self.tip
    }

    #[cfg(any(test, test_utilities))]
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        let mut chain = Vec::new();
        let mut curr = self.tip;
        while let Some(block) = self.hash_to_block.get(&curr) {
            chain.push(curr);
            if curr == Block::genesis().hash() { break; }
            curr = block.header.parent;
        }
        chain.reverse();
        chain
    }
}

#[cfg(any(test, test_utilities))]
mod tests {
    use super::*;
    use crate::block::test::generate_random_block;
    use crate::crypto::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());

    }
}
