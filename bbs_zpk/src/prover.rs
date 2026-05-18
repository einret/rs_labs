use crate::error::{AppError, Result};
use crate::traits::{FieldExtractor, FullExtractor, Signer};
use bbs::{pm_hidden, pm_revealed, prelude::*};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct ProofOfKnowledge {
    pub(crate) signature: SignatureProof,
    pub(crate) proof_request: ProofRequest,
    pub(crate) nonce: ProofNonce,
}

pub(crate) fn generate_selective_disclosure<
    F: std::fmt::Display,
    T: FieldExtractor<F> + FullExtractor,
>(
    claimant: &T,
    signature: &[u8; 112],
    signer: Arc<dyn Signer>,
    fields_to_reveal: &[F],
) -> Result<ProofOfKnowledge> {
    let nonce = Verifier::generate_proof_nonce();

    let mut index_to_reveal = Vec::new();
    for field in fields_to_reveal {
        let idx = claimant
            .find_field_index(field)
            .ok_or_else(|| AppError::InternalError {
                reason: format!("Field '{}' not found in identity", field),
            })?;
        index_to_reveal.push(idx);
    }

    let proof_request = Verifier::new_proof_request(&index_to_reveal, signer.public_key())?;

    let mut revealed_messages = Vec::new();
    let proof_messages: Vec<ProofMessage> = claimant
        .get_all_fields()
        .into_iter()
        .enumerate()
        .map(|(i, bytes)| {
            if index_to_reveal.contains(&i) {
                revealed_messages.push(SignatureMessage::hash(&bytes));
                pm_revealed!(bytes)
            } else {
                pm_hidden!(bytes)
            }
        })
        .collect();

    let sig_obj = Signature::from(*signature);

    let pok = Prover::commit_signature_pok(&proof_request, proof_messages.as_slice(), &sig_obj)?;

    let challenge = Prover::create_challenge_hash(&[pok.clone()], None, &nonce)?;
    let sig = Prover::generate_signature_pok(pok, &challenge)?;

    Ok(ProofOfKnowledge {
        signature: sig,
        proof_request,
        nonce,
    })
}
