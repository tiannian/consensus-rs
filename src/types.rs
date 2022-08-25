pub struct Voter<V, P> {
    pub voter_id: V,
    pub public_key: P,
}

pub struct VoterSet<V, P> {
    pub set: Vec<Voter<V, P>>,
}

pub struct VoteSign<S> {
    pub vs_idx: u64,
    pub sign: S,
}

pub struct VoteSignSet<S> {
    pub set: Vec<VoteSign<S>>,
}
