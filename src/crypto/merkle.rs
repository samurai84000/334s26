use super::hash::{Hashable, H256};
use ring::digest;
/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    levels: Vec<Vec<H256>>,
}

impl MerkleTree {
    fn hash_children(left: &H256, right: &H256) -> H256 {
        let mut ctx = digest::Context::new(&digest::SHA256);

        ctx.update(left.as_ref());
        ctx.update(right.as_ref());
        ctx.finish().into()
    }

    pub fn new<T>(data: &[T]) -> Self where T: Hashable, {
        assert!(!data.is_empty());

        let mut levels = Vec::new();
        let mut current_level: Vec<H256> = data.iter().map(|item| item.hash()).collect();
        levels.push(current_level.clone());

        while current_level.len() > 1 {
            if current_level.len() % 2 == 1 {
                let last = current_level.last().unwrap().clone();
                current_level.push(last);
            }

            let mut next_level = Vec::new();
            for i in (0..current_level.len()).step_by(2) {
                next_level.push(Self::hash_children(&current_level[i], &current_level[i + 1]));
            }
            current_level = next_level.clone();
            levels.push(next_level);
        }

        MerkleTree { levels }
    }

    pub fn root(&self) -> H256 {
        self.levels.last().expect("Tree should have levels")[0]
    }

    pub fn proof(&self, index: usize) -> Vec<H256> {
        let mut proof = Vec::new();
        let mut curr_idx = index;

        for i in 0..self.levels.len() - 1 {
            let level = &self.levels[i];
            let sibling_idx = if curr_idx % 2 == 0 { curr_idx + 1 } else { curr_idx - 1 };
            proof.push(level[sibling_idx]);
            curr_idx /= 2;
        }
        proof
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, _leaf_size: usize) -> bool {
    let mut curr_hash = *datum;
    let mut curr_idx = index;

    for sibling in proof {
        let mut ctx = digest::Context::new(&digest::SHA256);
        if curr_idx % 2 == 0 {
            ctx.update(curr_hash.as_ref());
            ctx.update(sibling.as_ref());
        } else {
            ctx.update(sibling.as_ref());
            ctx.update(curr_hash.as_ref());
        }
        curr_hash = ctx.finish().into();
        curr_idx /= 2;
    }
    curr_hash == *root
}

#[cfg(test)]
mod tests {
    use crate::crypto::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    macro_rules! gen_merkle_tree_large {
        () => {{
            vec![
                (hex!("0000000000000000000000000000000000000000000000000000000000000011")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000022")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000033")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000044")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000055")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000066")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000077")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000088")).into(),
            ]
        }};
    }
  
    #[test]
    fn root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn proof_tree_large() {
        let input_data: Vec<H256> = gen_merkle_tree_large!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(5);
  
        // We accept the proof in either the top-down or bottom-up order; you should stick to either of them.
        let expected_proof_bottom_up: Vec<H256> = vec![
            (hex!("c8c37c89fcc6ee7f5e8237d2b7ed8c17640c154f8d7751c774719b2b82040c76")).into(),
            (hex!("bada70a695501195fb5ad950a5a41c02c0f9c449a918937267710a0425151b77")).into(),
            (hex!("1e28fb71415f259bd4b0b3b98d67a1240b4f3bed5923aa222c5fdbd97c8fb002")).into(),
        ];
        let expected_proof_top_down: Vec<H256> = vec![
            (hex!("1e28fb71415f259bd4b0b3b98d67a1240b4f3bed5923aa222c5fdbd97c8fb002")).into(),  
            (hex!("bada70a695501195fb5ad950a5a41c02c0f9c449a918937267710a0425151b77")).into(),
            (hex!("c8c37c89fcc6ee7f5e8237d2b7ed8c17640c154f8d7751c774719b2b82040c76")).into(),
        ];
        assert!(proof == expected_proof_bottom_up || proof == expected_proof_top_down);
    }
    
    #[test]
    fn verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }
}