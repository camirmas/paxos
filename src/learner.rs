//! Learner

use message::AcceptedData;
use message::Message;
use message::Messenger;
use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::hash::Hash;
use std::sync::Arc;

/// Learners act as the replication factor for the protocol. Once a Client
/// request has been agreed on by the Acceptors, the Learner may take action
/// (i.e.: execute the request and send a response to the client). To improve
/// availability of processing, additional Learners can be added.
pub struct Learner<T> {
    /// `Proposer`'s ID
    pub id: u64,
    /// `Messenger` specifying communication with other nodes
    pub messenger: Option<Box<Messenger<T>>>,
    /// The last proposal that was accepted
    pub last_accepted_n: u64,
    /// Accepted messages received (proposal_n => data)
    pub accepted_received: HashMap<u64, HashSet<AcceptedData<T>>>,
    /// The last accepted value
    pub value: Option<Arc<T>>,
    /// Quorum size
    pub quorum: u8,
}

impl<T> Learner<T>
where
    T: Hash + Eq,
{
    pub fn new(id: u64, quorum: u8) -> Self {
        Self {
            id,
            messenger: None,
            last_accepted_n: 0,
            accepted_received: HashMap::new(),
            value: None,
            quorum,
        }
    }

    /// Receives an `Accepted` message from an `Acceptor`.
    pub fn receive_accepted(&mut self, msg: Message<T>) {
        if let Message::Accepted(data) = msg {
            let id = data.id;
            if id == self.last_accepted_n {
                if let Some(ref val) = self.value {
                    if *val != data.value {
                        panic!("Value mismatch for proposal {}", id);
                    }
                }
            }

            self.accepted_received.entry(id).or_insert(HashSet::new());

            self.accepted_received.get_mut(&id).unwrap().insert(data);

            if self.accepted_received.get(&id).unwrap().len() == self.quorum as usize {
                self.value = Some(
                    self.accepted_received
                        .get(&id)
                        .unwrap()
                        .iter()
                        .next()
                        .unwrap()
                        .value
                        .clone(),
                );
                self.last_accepted_n = id;
                if let Some(ref mut messenger) = self.messenger {
                    messenger.on_resolution(id, self.value.clone().unwrap());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learner_new() {
        let l: Learner<u64> = Learner::new(1, 7);

        assert_eq!(l.id, 1);
        assert!(l.messenger.is_none());
        assert_eq!(l.last_accepted_n, 0);
        assert!(l.value.is_none());
        assert_eq!(l.accepted_received, HashMap::new());
    }

    #[test]
    fn learner_receive_accepted() {
        let mut l: Learner<u64> = Learner::new(1, 7);

        let id = 1;
        let msg = Message::Accepted(AcceptedData {
            id,
            value: Arc::new(10),
            from: 0,
        });

        l.receive_accepted(msg);

        assert_eq!(l.value, None);
        assert_eq!(l.accepted_received.get(&id).unwrap().len(), 1);

        for i in 1..l.quorum {
            let msg = Message::Accepted(AcceptedData {
                id: 1,
                value: Arc::new(10),
                from: i as u64,
            });
            l.receive_accepted(msg);
        }

        assert_eq!(l.last_accepted_n, 1);
        assert_eq!(l.value, Some(Arc::new(10)));
    }

    #[test(should_panic)]
    fn learner_receive_accepted_mismatch() {
        let mut l: Learner<u64> = Learner::new(1, 7);

        let id = 1;
        let msg = Message::Accepted(AcceptedData {
            id,
            value: Arc::new(10),
            from: 0,
        });

        l.receive_accepted(msg);

        let msg = Message::Accepted(AcceptedData {
            id: 1,
            value: Arc::new(8), // conflicting value
            from: 1 as u64,
        });
        l.receive_accepted(msg);
    }
}
