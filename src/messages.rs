pub struct BroadcastPropose<I, H> {
    pub epoch_id: I,
    pub epoch_hash: H,
}

pub struct ResponsePropose<I, H> {
    pub epoch_id: I,
    pub epoch_hash: H,
}

pub struct BroadcastCommit<I, H> {
    pub epoch_id: I,
    pub epoch_hash: H,
}

pub enum Packet<I, H> {
    BroadcastPropose(BroadcastPropose<I, H>),
    ResponsePropose(ResponsePropose<I, H>),
    BroadcastCommit(BroadcastCommit<I, H>),
}

impl<I, H> Packet<I, H> {
    pub fn is_broadcast_propose(&self) -> bool {
        matches!(self, Packet::BroadcastPropose(_))
    }

    pub fn is_response_propose(&self) -> bool {
        matches!(self, Packet::ResponsePropose(_))
    }

    pub fn response_from_broadcast_propose(pkt: BroadcastPropose<I, H>) -> Self {
        Self::ResponsePropose(ResponsePropose {
            epoch_id: pkt.epoch_id,
            epoch_hash: pkt.epoch_hash,
        })
    }
}
