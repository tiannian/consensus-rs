# consensus-rs

> A finalized consensus for blockchain and other distributed system.

## Support Algorithms

- [X] [BRaft](docs/BRaft.md)
- [ ] [BPBFT](docs/BPBFT.md)
- [ ] Tendermint

## Concept

### Node

Nodes are consensus participants. Eventually each node will deterministically obtain a consistent state. All nodes are connected by a P2P network to form a consensus network, and all nodes are peer-to-peer

#### Node Role

Node roles represent the behavior of nodes in the consensus process.

- Voter: Do consensus among these node 
   - Proposer: Propose epoch
   - Follower: Accept and check epoch, Have chance to become a Proposer.
- Observer: Only accept and check epoch.

### SMR

`cosensus-rs` is a SMR (State Machine Replication) Framework. Through this framework, the state consistency of multiple nodes is achieved. This framework uses a similar blockchain to complete SMR.

#### Epoch

In this framework, timeline is divided into multiple `Epoch`s.  Each `Epoch` represents the state diff at a segment of time. 

Each node use same logic to execute epoch following the sequence of epoch to build state. From the genesis `Epoch` to a `Epoch` represents the final state at this point in time.

Each epoch is identified with `Epoch ID`. It's A increasing number (may sparse).

> image here. State diff and State

- **Epoch**: State diff for a segment of time.
- **Genesis Epoch**: The first epoch.
- **Epoch ID**: A increasing number to identify Epoch.

#### Epoch Hash

The state diff for each `Epoch` is represented using data with any length. This data is hashed as a `Epoch Hash`. In order to ensure that the `Epoch Hash` is unique, the hash calculation should include the previous `Epoch Hash`.

#### Sparse Epoch

Epoch IDs do not have to be strictly auto-incrementing. It can be sparse.  Just make sure the numbers increased based on time.

### Relaxed Stable Network

This framework is based on a relaxed static network environment. This means no protection against sybil attacks.

**Static** means each node know the set of all the nodes.

**Relaxed** means the set of all the nodes can change. Each `Epoch` based on the set of node at previous `Epoch`.

### Fault Tolerance Type

- $n$: Total number of all the nodes.
- $t$: The number of fault nodes.

This framework support two type:

- CFT: Crash Fault Tolerance. $2t < n$
- BFT: Byzantine Fault Tolerance. $3t < n$

The framework supports weight-based fault tolerance. With weight:

- $n$: Total weight of all the nodes.
- $t$: The weight of fault nodes.

## Design

### Consensus Triple

- Epoch: Which Epoch is consensus.
- Round: How many round change caused by fault.
- Step: Step of consensus, work with node role. For different algorithm.

### Node ID and Identity

Each Node have an NodeID and Keypair(s).

NodeID must be a unique, recommend to compute this use hash.

This framework does not check `PublicKey` with `NodeID`.

#### Voter Set and Weight

`Voter Set` is an array of voters. Each voter include these field:

- NodeID
- PublicKey
- Weight

When `Epoch` is consensus, It use the latest `Voter Set`.

When an `Epoch` reach consensus, the framework checks and updates the voter set.
The `NodeID` must be unique, or fallback to latest `Voter Set`

### Network Layer

The framework will make signature for all outcoming packet, and verify all the
signature of incoming packet.

The Network Layer only have 3 method. The framework use these methods to complete P2P communication.

```rust
fn node_id() -> NodeId;

async fn send_unsigned(node_id: Option<&NodeId>, pkt: Packet);

async fn recv() -> (Packet, NodeId);
```

Use `send_unsigned` to send an unsigned packet. The Network Layer use this node's secret key to
sign this packet. 

If `node_id` is `None`, the Network Layer will broadcast this packet.
The Network Layer broadcast to all the Voter node first.
It will broadcast to Non-Voter node also.

If is a `node_id`, send packet to the specific code.

Use `recv` to receive packet from other node. If message from Voter, the node id also got.

#### Packet

### Consensus Layer

#### Latest Epoch

#### Epoch Check

#### Proposer Election

#### Byzantine Evidence

### Application

#### Propose Epoch

#### Enter Step

#### Commit
