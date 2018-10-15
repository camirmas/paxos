extern crate paxos_rust;

use paxos_rust::{Acceptor, Message, Messenger, Proposer};
use std::hash::Hash;
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;

/// A `Messenger` that utilizes Channels to pass values between threads.
pub struct ChannelMessenger<T> {
    pub sender: Sender<Message<T>>,
}

impl<T> Messenger<T> for ChannelMessenger<T>
where
    T: Eq + Hash + Clone,
{
    fn send_prepare(&mut self, msg: Message<T>) {
        println!("PREPARE");
        self.sender.send(msg).unwrap();
    }

    fn send_promise(&mut self, msg: Message<T>) {
        println!("PROMISE");
        self.sender.send(msg).unwrap();
    }

    fn send_accept(&mut self, msg: Message<T>) {
        println!("ACCEPT");
        self.sender.send(msg).unwrap();
    }

    fn send_accepted(&mut self, msg: Message<T>) {
        println!("ACCEPTED");
        self.sender.send(msg).unwrap();
    }

    fn on_resolution(&mut self, _proposal_n: u64, _value: Arc<T>) {}
}

#[test]
/// Should demonstrate sending messages between Proposers and Acceptors,
/// eventually reaching consensus on the proposed value.
fn basic_paxos() {
    let (acc_sender, acc_receiver) = mpsc::channel();
    let (proposer_sender, proposer_receiver) = mpsc::channel();

    thread::spawn(move || {
        let mut acc: Acceptor<u64> = Acceptor::new(1);
        let messenger = ChannelMessenger {
            sender: proposer_sender,
        };

        acc.messenger = Some(Box::new(messenger));

        loop {
            if let Ok(msg) = acc_receiver.recv() {
                match msg {
                    Message::Prepare(_) => acc.receive_prepare(&msg),
                    Message::Accept(_) => acc.receive_accept(&msg),
                    _ => {}
                }
            }
        }
    });

    let p_thread = thread::spawn(move || {
        let mut proposer: Proposer<u64> = Proposer::new(1);
        let messenger = ChannelMessenger { sender: acc_sender };
        proposer.messenger = Some(Box::new(messenger));
        proposer.quorum = 1;

        proposer.prepare(10);

        loop {
            if proposer.last_accepted_n == 1 {
                break;
            }
            if let Ok(msg) = proposer_receiver.recv() {
                match msg {
                    Message::Promise(_) => proposer.receive_promise(msg),
                    Message::Accepted(_) => proposer.receive_accepted(msg),
                    _ => {}
                }
            }
        }
    });

    assert!(p_thread.join().is_ok());
}
