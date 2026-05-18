# BBS+ Zero-Knowledge Proofs (ZKP) Lab

This sub-project is an exploratory laboratory aimed at understanding the mechanics and mathematics behind **BBS+ Signatures** and **Zero-Knowledge Proofs (ZKP)** using Rust, [check the source code here](./src/main.rs).

The goal here is not to build a production-ready cryptographic library, but rather to demystify how these advanced cryptographic concepts work under the hood. It serves as a personal sandbox to map theoretical mathematics to practical Rust code using bilinear pairings.

## Documentation

To help make sense of the complex concepts involved, I've written down my notes and understanding in the `docs/` directory. If you are new to this (like I am!), I highly recommend reading them in order:

1. **[Mathematics 101](docs/00_mathematics_101.md)**: Back-to-basics introduction to Elliptic Curves and Field Extensions.
2. **[Architecture & Flow](docs/01_architecture_flow.md)**: A high-level overview of the components in this repository (Issuer, Identity, Prover, Verifier) and how they interact to issue and verify a zero-knowledge proof.
3. **[Mathematics of BBS+](docs/02_mathematics_of_bbs.md)**: A deeper dive into how BBS+ signatures and bilinear pairings actually function, mapped directly to the Rust codebase.

## Disclaimer

I'm learning these concepts and I'm not a professional cryptographer or mathematician. These implementations and notes reflect my current understanding and are primarily for educational purposes. There may be inaccuracies!
