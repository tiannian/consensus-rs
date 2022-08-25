pub struct BroadcastPropose<I, H> {
    pub epoch_id: I,
    pub epoch_hash: H,
}

pub struct ResponsePropose<I, H, S> {
    pub epoch_id: I,
    pub epoch_hash: H,
    pub sign_idx: u64,
    pub sign: S,
}

pub enum SendPacket<I, H> {
    BroadcastPropose(BroadcastPropose<I, H>),
}

pub enum RecvPacket<I, H, S> {
    ResponsePropose(ResponsePropose<I, H, S>),
}
