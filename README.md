# consensus-rs

## Support Algorithms

- [ ] BRaft
- [ ] BPBFT
- [ ] Tendermint

## Concept

### Node

Nodes are consensus participants. Eventually each node will deterministically obtain a consistent state. All nodes are connected by a P2P network to form a consensus network, and all nodes are peer-to-peer

#### Node Role

Node roles represent the behavior of nodes in the consensus process.

- Voter: Do consensus among these node 
   - Proposer: Propose epoch
   - Follower: Accept and check epoch, Have change to become a Proposer.
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

Epoch IDs do not have to be strictly auto-incrementing. It can be sparse.  Just make sure the numbers increase over time.

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

### Comsensus Triple

- Epoch: Which Epoch is consensus.
- Round: How many round change caused by fault.
- Step: Step of consensus, work with node role. For different algorithm.

### Node ID and Identity

Each Node have an ID and Keypair(s).

#### Voter Set and Weight

### Network and Signature

#### Packet

#### P2P Network.

##### Incoming Packet

##### Outcoming Packet

### Consensus and Proposer Election

#### Latest Epoch

#### Epoch Check

#### Proposer Election

#### Byzantine Evidence

### Application

#### Propose Epoch

#### Enter Step

#### Commit
