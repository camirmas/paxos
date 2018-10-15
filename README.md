# :love_letter: Paxos [![Latest Version]][crates.io]

[Latest Version]: https://img.shields.io/crates/v/paxos-rust.svg
[crates.io]: https://crates.io/crates/paxos-rust

> A (Rust flavored) lightweight implementation of the Paxos Consensus Algorithm

### Introduction

There are plenty of strong resources on this topic that can explain it better than I can:

1. The OG, [Paxos Made Simple](https://www.cs.utexas.edu/users/lorenzo/corsi/cs380d/past/03F/notes/paxos-simple.pdf) - by Leslie Lamport
2. [Wikipedia](https://en.wikipedia.org/wiki/Paxos_(computer_science))
3. An [article](https://understandingpaxos.wordpress.com/) and corresponding [repo](https://github.com/cocagne/paxos) - by Tom Cocagne

### Usage and Examples

See [docs](https://docs.rs/paxos-rust/0.1.1/paxos_rust/)

### Next steps

- Improve error handling
- Add `Nack` messages + handling
- Write integration tests for failure scenarios
