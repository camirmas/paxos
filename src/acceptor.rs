//! Acceptor

use message::{AcceptedData, Message, Messenger, PromiseData};
use std::sync::Arc;

/// The Acceptors act as the fault-tolerant "memory" of the protocol. Acceptors
/// are collected into groups called Quorums. Any message sent to an Acceptor
/// must be sent to a Quorum of Acceptors. Any message received from an Acceptor
/// is ignored unless a copy is received from each Acceptor in a Quorum.
pub struct Acceptor<T> {
    /// `Acceptor`'s ID
    pub id: u64,
    /// The highest proposal number promised
    pub proposal_n: u64,
    /// The currently promised value
    pub value: Option<Arc<T>>,
    /// `Messenger` specifying communication with other nodes
    pub messenger: Option<Box<Messenger<T>>>,
}

impl<T> Acceptor<T> {
    /// Creates a new `Acceptor`.
    pub fn new(id: u64) -> Self {
        Self {
            id,
            proposal_n: 0,
            value: None,
            messenger: None,
        }
    }

    /// Receives a `Prepare` message from a `Proposer`.
    pub fn receive_prepare(&mut self, msg: &Message<T>) {
        if let Message::Prepare(data) = msg {
            if data.id > self.proposal_n {
                self.proposal_n = data.id;
                let promise = Message::Promise(PromiseData {
                    id: self.proposal_n,
                    value: self.value.clone(),
                    from: self.id,
                });
                if let Some(ref mut messenger) = self.messenger {
                    messenger.send_promise(promise);
                }
            }
        }
    }

    /// Receives an `Accept` message from a `Proposer`.
    pub fn receive_accept(&mut self, msg: &Message<T>) {
        if let Message::Accept(data) = msg {
            if data.id >= self.proposal_n {
                self.value = Some(data.value.clone());
                self.proposal_n = data.id;
                let accepted = Message::Accepted(AcceptedData {
                    id: self.proposal_n,
                    value: data.value.clone(),
                    from: self.id,
                });
                if let Some(ref mut messenger) = self.messenger {
                    messenger.send_accepted(accepted);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use message::{AcceptData, ProposalData};

    #[test]
    fn acceptor_new() {
        let a: Acceptor<u64> = Acceptor::new(1);

        assert_eq!(a.id, 1);
        assert_eq!(a.proposal_n, 0);
        assert_eq!(a.value, None);
        assert!(a.messenger.is_none());
    }

    #[test]
    fn acceptor_receive_prepare() {
        let mut a: Acceptor<u64> = Acceptor::new(1);

        let msg = Message::Prepare(ProposalData { id: 8 });

        a.receive_prepare(&msg);

        assert_eq!(a.proposal_n, 8);

        // ignore proposals less than N
        let msg = Message::Prepare(ProposalData { id: 6 });

        a.receive_prepare(&msg);

        assert_eq!(a.proposal_n, 8);
    }

    #[test]
    fn acceptor_receive_accept() {
        let mut a: Acceptor<u64> = Acceptor::new(1);

        let msg = Message::Accept(AcceptData {
            id: 3,
            value: Arc::new(60),
        });

        a.receive_accept(&msg);

        assert_eq!(a.value, Some(Arc::new(60)));
        assert_eq!(a.proposal_n, 3);

        // ignore Accept messages less than N

        let msg = Message::Accept(AcceptData {
            id: 2,
            value: Arc::new(60),
        });

        a.receive_accept(&msg);

        assert_eq!(a.value, Some(Arc::new(60)));
        assert_eq!(a.proposal_n, 3);
    }
}
