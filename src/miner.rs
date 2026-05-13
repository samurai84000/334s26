use crate::network::server::Handle as ServerHandle;
use crate::blockchain::Blockchain;
use crate::block::{Block, Header, Content};
use crate::crypto::hash::{H256, Hashable};
use crate::crypto::merkle::MerkleTree;

use log::info;
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time::{self, SystemTime, UNIX_EPOCH};
use std::thread;
use std::sync::{Arc, Mutex};

enum ControlSignal {
    Start(u64), // lambda
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>, // Add this
}
#[derive(Clone)]
pub struct Handle {
    control_chan: Sender<ControlSignal>,
}

pub fn new(server: &ServerHandle, blockchain: &Arc<Mutex<Blockchain>>) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    (
        Context {
            control_chan: signal_chan_receiver,
            operating_state: OperatingState::Paused,
            server: server.clone(),
            blockchain: Arc::clone(blockchain), // Clone the Arc
        },
        Handle { control_chan: signal_chan_sender },
    )
}

impl Handle {
    pub fn exit(&self) {
        let _ = self.control_chan.send(ControlSignal::Exit);
    }

    pub fn start(&self, lambda: u64) {
        let _ = self.control_chan.send(ControlSignal::Start(lambda));
    }
}

impl Context {
    pub fn start(mut self) {
        thread::spawn(move || {
            self.miner_loop();
        });
        info!("Miner initialized into paused mode");
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                info!("Miner shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i) => {
                info!("Miner starting in continuous mode with lambda {}", i);
                self.operating_state = OperatingState::Run(i);
            }
        }
    }

fn miner_loop(&mut self) {
        loop {
            // ... (Keep the existing match self.operating_state logic)

            if let OperatingState::Run(i) = self.operating_state {
                // SIMPLE MINING LOGIC
                let (parent, difficulty) = {
                    let bc = self.blockchain.lock().unwrap();
                    (bc.tip(), [0u8; 32].into()) // Use a target for mining
                };

                let transactions = vec![]; // Empty for now
                let mut block = Block {
                    header: Header {
                        parent,
                        nonce: rand::random(),
                        difficulty,
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
                        merkle_root: MerkleTree::new(&transactions).root(),
                    },
                    content: Content { transactions },
                };

                // Try to solve the puzzle
                if block.hash() <= difficulty {
                    let mut bc = self.blockchain.lock().unwrap();
                    bc.insert(&block);
                    info!("Mined a block! Hash: {}", block.hash());
                }

                if i != 0 {
                    thread::sleep(time::Duration::from_micros(i));
                }
            }
        }
    }
}