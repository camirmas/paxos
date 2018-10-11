//! Proposer

use message::{AcceptData, AcceptedData, Message, Messenger, PromiseData, ProposalData};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;

/// A Proposer advocates a client request, attempting to convince the Acceptors
/// to agree on it, and acting as a coordinator to move the protocol forward
/// when conflicts occur.
pub struct Proposer<T> {
    /// `Proposer`'s ID
    pub id: u64,
    /// `Messenger` specifying communication with other nodes
    pub messenger: Option<Box<Messenger<T>>>,
    /// The proposed value
    pub value: Option<Arc<T>>,
    /// The highest proposal number seen
    pub proposal_n: u64,
    /// The last proposal that was accepted
    pub last_accepted_n: u64,
    /// Promises received (proposal_n => data)
    pub promises_received: HashMap<u64, HashSet<PromiseData<T>>>,
    /// Accepted messages received (proposal_n => data)
    pub accepted_received: HashMap<u64, HashSet<AcceptedData<T>>>,
    /// The minimum number of `Acceptor`s needed to continue
    pub quorum: u8,
}

impl<T: 'static> Proposer<T>
where
    T: Eq + Hash + Clone,
{
    /// Creates a new `Proposer`.
    pub fn new(id: u64) -> Self {
        Self {
            id,
            value: None,
            messenger: None,
            proposal_n: 0,
            last_accepted_n: 0,
            promises_received: HashMap::new(),
            accepted_received: HashMap::new(),
            quorum: 7,
        }
    }

    /// The first phase. Creates a proposal.
    pub fn prepare(&mut self, value: T) {
        self.value = Some(Arc::new(value));
        self.proposal_n += 1;
        self.promises_received
            .insert(self.proposal_n, HashSet::new());
        self.accepted_received
            .insert(self.proposal_n, HashSet::new());
        let prepare = Message::Prepare(ProposalData { id: self.id });

        if let Some(ref mut messenger) = self.messenger {
            messenger.send_prepare(prepare);
        }
    }

    /// Receives a `Promise` message from an `Acceptor`.
    pub fn receive_promise(&mut self, msg: Message<T>) {
        if let Message::Promise(data) = msg {
            self.promises_received
                .get_mut(&data.id)
                .unwrap()
                .insert(data);

            if self.promises_received.get(&self.proposal_n).unwrap().len() == self.quorum as usize {
                self.accept();
            }
        }
    }

    /// The second phase. Sets a value for the proposal, and builds an `Accept` request.
    pub fn accept(&mut self) {
        let mut promises_vec = self
            .promises_received
            .get_mut(&self.proposal_n)
            .unwrap()
            .iter()
            .filter(|p| p.value.is_some())
            .collect::<Vec<_>>();

        promises_vec.sort_by(|a, b| b.id.cmp(&a.id));

        self.value = Some({
            if promises_vec.len() == 0 {
                self.value.clone().unwrap()
            } else {
                promises_vec[0].value.clone().unwrap()
            }
        });
        let msg = Message::Accept(AcceptData {
            id: self.proposal_n,
            value: self.value.clone().unwrap(),
        });

        if let Some(ref mut messenger) = self.messenger {
            messenger.send_accept(msg);
        }
    }

    /// Receives an `Accepted` message from an `Acceptor`.
    pub fn receive_accepted(&mut self, msg: Message<T>) {
        if let Message::Accepted(data) = msg {
            self.accepted_received
                .get_mut(&data.id)
                .unwrap()
                .insert(data);

            if self.accepted_received.get(&self.proposal_n).unwrap().len() == self.quorum as usize {
                if let Some(ref mut messenger) = self.messenger {
                    self.last_accepted_n = self.proposal_n;
                    messenger.on_resolution(self.proposal_n, self.value.clone().unwrap());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposer_new() {
        let p: Proposer<u64> = Proposer::new(1);

        assert_eq!(p.id, 1);
        assert_eq!(p.proposal_n, 0);
        assert_eq!(p.value, None);
        assert!(p.messenger.is_none());
        assert_eq!(p.promises_received.len(), 0);
        assert_eq!(p.accepted_received.len(), 0);
        assert_eq!(p.quorum, 7);
    }

    #[test]
    fn proposer_prepare() {
        let mut p: Proposer<u64> = Proposer::new(1);

        p.prepare(60);

        assert_eq!(p.proposal_n, 1);

        assert_eq!(p.value, Some(Arc::new(60)));
    }

    #[test]
    fn proposer_receive_promise() {
        let mut p: Proposer<u64> = Proposer::new(1);

        p.prepare(60);

        let msg = Message::Promise(PromiseData {
            id: 1,
            value: None,
            from: 2,
        });

        p.receive_promise(msg);

        assert_eq!(p.promises_received.len(), 1);
        assert!(p.promises_received.get(&1).is_some());
    }

    #[test]
    fn proposer_accept() {
        let mut p: Proposer<u64> = Proposer::new(1);

        p.prepare(60);

        // Receive a Promise

        let msg = Message::Promise(PromiseData {
            id: 1,
            value: None,
            from: 2,
        });

        p.receive_promise(msg);

        p.accept();

        assert_eq!(p.value, Some(Arc::new(60)));

        // Receive another Promise that has an existing value for that proposal.

        let msg = Message::Promise(PromiseData {
            id: 1,
            value: Some(Arc::new(25)),
            from: 2,
        });

        p.receive_promise(msg);

        p.accept();

        assert_eq!(p.value, Some(Arc::new(25)));
    }

    #[test]
    fn proposer_receive_accepted() {
        let mut p: Proposer<u64> = Proposer::new(1);

        p.prepare(60);

        let msg = Message::Accepted(AcceptedData {
            id: 1,
            value: Arc::new(60),
            from: 2,
        });

        p.receive_accepted(msg);

        assert_eq!(p.accepted_received.len(), 1);
        assert!(p.accepted_received.get(&1).is_some());
    }
}
