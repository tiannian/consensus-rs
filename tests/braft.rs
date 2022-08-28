use braft_test::{SingleApp, SingleConsensus, SingleNetwork};
use consensus_rs::algorithm::BRaft;

mod braft_test;
mod utils;

#[test]
fn signle_node() {
    utils::init();

    let network = SingleNetwork::new();
    let app = SingleApp::new();
    let consensus = SingleConsensus::new(vec![1]);

    let mut braft = BRaft::new(network, consensus, app);

    smol::block_on(async move {
        for _ in 0..30 {
            braft.do_tick().await.unwrap();
        }
    })
}

#[test]
fn signle_follower_node() {
    utils::init();

    let network = SingleNetwork::new();
    let app = SingleApp::new();
    let consensus = SingleConsensus::new(vec![2]);

    let mut braft = BRaft::new(network, consensus, app);

    smol::block_on(async move {
        for _ in 0..30 {
            braft.do_tick().await.unwrap();
        }
    })
}

#[test]
fn two_node() {}
