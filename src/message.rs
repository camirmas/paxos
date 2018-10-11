//! Describes Paxos messages

use std::sync::Arc;

/// A message sent between nodes
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Message<T> {
    Prepare(ProposalData),
    Promise(PromiseData<T>),
    Accept(AcceptData<T>),
    Accepted(AcceptedData<T>),
    Nack,
}

/// Proposal data (Proposer -> Acceptor)
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ProposalData {
    pub id: u64,
}

/// Promise data (Acceptor -> Proposer)
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PromiseData<T> {
    pub id: u64,
    pub value: Option<Arc<T>>,
    pub from: u64,
}

/// Accept data (Proposer -> Acceptor)
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AcceptData<T> {
    pub id: u64,
    pub value: Arc<T>,
}

/// Accepted data (Acceptor -> Proposer)
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AcceptedData<T> {
    pub id: u64,
    pub value: Arc<T>,
    pub from: u64,
}

pub trait Messenger<T> {
    fn send_prepare(&mut self, msg: Message<T>);

    fn send_promise(&mut self, msg: Message<T>);

    fn send_accept(&mut self, msg: Message<T>);

    fn send_accepted(&mut self, msg: Message<T>);

    fn on_resolution(&mut self, proposal_n: u64, value: Arc<T>);
}
