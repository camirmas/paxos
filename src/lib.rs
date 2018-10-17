//! A lightweight implementation of the Paxos Consensus Algorithm.

pub mod acceptor;
pub mod learner;
pub mod message;
pub mod proposer;

pub use acceptor::*;
pub use learner::*;
pub use message::*;
pub use proposer::*;
