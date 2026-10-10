mod error;
mod identity;
mod issuer;
mod prover;
mod traits;

use bbs::prelude::*;
use error::{AppError, Result};
use identity::{IdentityBody, IdentityField};
use prover::generate_selective_disclosure;
use traits::FieldExtractor;

fn main() -> Result<()> {
    let issuer = issuer::MyIssuer::new(3)?;
    let body = IdentityBody {
        name: "Alice".to_string(),
        is_major: true,
        address: "42-4567 crypto world".to_string(),
    };
    let alice = issuer.sign_identity(&body)?;

    println!("Identity: {:?}", alice);
    println!();
    println!("Is {} a major?", alice.body().name);

    let proof = generate_selective_disclosure(
        &alice,
        alice.signature(),
        alice.issuer_pk(),
        &[IdentityField::IsMajor],
    )?;

    println!("lets verify the proof and signature");
    let revealed_messages =
        Verifier::verify_signature_pok(&proof.proof_request, &proof.signature, &proof.nonce)?;

    let expected_value = alice
        .get_field(&IdentityField::IsMajor)
        .expect("Field IsMajor not found");

    let expected_hash = SignatureMessage::hash(&expected_value);
    let expected_revealed_messages = vec![expected_hash];

    if revealed_messages != expected_revealed_messages {
        return Err(AppError::VerificationFailed {
            reason: format!(
                "expected {:?} got {:?}",
                expected_revealed_messages, revealed_messages
            ),
        });
    }

    println!(
        "Proof verified successfully, revealed message {:?}, expected message {:?}",
        revealed_messages, expected_revealed_messages
    );
    Ok(())
}
