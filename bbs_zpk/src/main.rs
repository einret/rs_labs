mod error;
mod identity;
mod issuer;
mod prover;
mod traits;

use anyhow::Result;
use bbs::prelude::*;
use identity::{Identity, IdentityField};
use prover::generate_selective_disclosure;
use std::sync::Arc;
use traits::FieldExtractor;

fn main() -> Result<()> {
    let signer = issuer::MyIssuer::new(3).unwrap();
    let alice = Identity::new(
        "Alice".to_string(),
        true,
        "42-4567 crypto world".to_string(),
        Arc::new(signer),
    )?;

    println!("Identity: {:?}", alice);
    println!();
    println!("Is {} a major?", alice.name());

    let proof = generate_selective_disclosure(
        &alice,
        alice.signature(),
        alice.signer(),
        &[IdentityField::IsMajor],
    )?;

    println!("lets verify the proof and signature");
    let revealed_messages =
        Verifier::verify_signature_pok(&proof.proof_request, &proof.signature, &proof.nonce)
            .map_err(|e| anyhow::anyhow!("Proof verification failed: {}", e))?;

    let expected_value = alice
        .get_field(&IdentityField::IsMajor)
        .expect("Field IsMajor not found");

    let expected_hash = SignatureMessage::hash(&expected_value);
    let expected_revealed_messages = vec![expected_hash];

    if revealed_messages != expected_revealed_messages {
        anyhow::bail!(
            "Proof verification failed expected {:?} got {:?}",
            expected_revealed_messages,
            revealed_messages
        );
    }

    println!(
        "Proof verified successfully, revealed message {:?}, expected message {:?}",
        revealed_messages, expected_revealed_messages
    );

    Ok(())
}
