# Architecture & Flow of the BBS+ Zero-Knowledge Proof System

This poc is to understand the flow of a BBS+ ZKP implementation. The goal of this system is to allow a user (Claimant/Prover) to obtain a signature on multiple attributes from a trusted authority (`MyIssuer`), and then later prove possession of that signature to a third party (Verifier) while *only* revealing a specific subset of those attributes.

## System Components

1. **Issuer (`issuer.rs`)**: Represents a trusted authority. It generates cryptographic keys and signs a user's data. It relies on the `FullExtractor` trait to extract the raw bytes of the data it needs to sign.
2. **Identity (`identity.rs`)**: Represents the user's data model:
   - **`IdentityBody`**: The raw claims/attributes (`name`, `is_major`, `address`). Implements `FullExtractor` to provide attribute bytes for signing, and `FieldExtractor<IdentityField>` to map field enum variants to their exact positional index.
   - **`Identity`**: The issued credential held by the user. It contains `IdentityBody`, the compressed signature (`[u8; 112]`), and the Issuer's `PublicKey`. Crucially, it does **not** contain the Issuer instance or the secret signing key.
3. **Prover (`prover.rs`)**: Represents the client-side proof generation logic (`generate_selective_disclosure`). It takes the user's attributes, signature, and selectively discloses only the requested fields to generate a zero-knowledge Proof of Knowledge (PoK).
4. **Verifier (`main.rs`)**: Represents the entity requesting and checking the proof. It validates the mathematical proof against the Issuer's public key without ever learning the hidden fields.

## Sequence Diagram

Below is the step-by-step interaction between the different components of the system.

```mermaid
sequenceDiagram
    participant I as Issuer
    participant A as Alice (Identity)
    participant P as Prover (generate_selective_disclosure)
    participant V as Verifier

    %% Phase 1: Setup & Issuance
    Note over I, A: Phase 1: Issuance (See Math Doc Step 2 & 3)
    I->>I: Generate Keypair (Public Key, Secret Key)
    A->>I: Request Signature on IdentityBody (`issuer.sign_identity(&body)`)
    I->>I: Extracts bytes via `FullExtractor::get_all_fields()`
    I->>I: Hash all fields & Sign using Secret Key
    I-->>A: Return issued Identity (Body + Signature + Issuer PK)
    Note over A: Alice holds Identity (Body + Signature + Issuer PK)

    %% Phase 2: Selective Disclosure
    Note over A, P: Phase 2: Proof Generation (See Math Doc Step 4)
    V->>A: "Prove to me you are major (IsMajor)"
    A->>P: Call `generate_selective_disclosure` with Identity & Issuer Public Key
    P->>A: Extract indices via `FieldExtractor::find_field_index(IsMajor)`
    P->>A: Extract raw bytes for revealed fields
    P->>P: Build ProofRequest using Issuer Public Key
    P->>P: Generate Proof of Knowledge (PoK) hiding Name & Address
    P-->>V: Return `ProofOfKnowledge` (SignatureProof, ProofRequest, Nonce)

    %% Phase 3: Verification
    Note over V: Phase 3: Verification (See Math Doc Step 5)
    V->>V: 1. Cryptographic Check: `Verifier::verify_signature_pok(...)`
    Note right of V: Ensures the signature is valid and hasn't been forged.
    V-->>V: Returns `Ok([SignatureMessage])`
    V->>V: 2. Business Logic Check
    Note right of V: Checks if the revealed hash matches hash(b"true")
    V->>V: Success!
```
