use crate::error::{AppError, CompressedSignature, Result};
use crate::traits::{FieldExtractor, FullExtractor};
use bbs::{pm_hidden, pm_revealed, prelude::*};

#[derive(Debug)]
pub(crate) struct ProofOfKnowledge {
    pub(crate) signature: SignatureProof,
    pub(crate) proof_request: ProofRequest,
    pub(crate) nonce: ProofNonce,
}

pub(crate) fn generate_selective_disclosure<
    F: std::fmt::Debug,
    T: FieldExtractor<F> + FullExtractor,
>(
    claimant: &T,
    signature: &CompressedSignature,
    issuer_pk: &PublicKey,
    fields_to_reveal: &[F],
) -> Result<ProofOfKnowledge> {
    let nonce = Verifier::generate_proof_nonce();

    let mut index_to_reveal = Vec::new();
    for field in fields_to_reveal {
        let idx = claimant
            .find_field_index(field)
            .ok_or_else(|| AppError::InternalError {
                reason: format!("Field '{:?}' not found in identity", field),
            })?;
        index_to_reveal.push(idx);
    }

    let proof_request = Verifier::new_proof_request(&index_to_reveal, issuer_pk)?;

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

    let challenge = Prover::create_challenge_hash(std::slice::from_ref(&pok), None, &nonce)?;
    let sig = Prover::generate_signature_pok(pok, &challenge)?;

    Ok(ProofOfKnowledge {
        signature: sig,
        proof_request,
        nonce,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{Identity, IdentityBody, IdentityField};
    use crate::issuer;

    fn create_identity() -> Identity {
        let issuer = issuer::MyIssuer::new(3).unwrap();
        let body = IdentityBody {
            name: "Carol".to_string(),
            is_major: true,
            address: "777 Secret Blvd".to_string(),
        };
        issuer.sign_identity(&body).unwrap()
    }

    #[test]
    fn test_end_to_end_selective_disclosure() {
        let identity = create_identity();

        let proof = generate_selective_disclosure(
            &identity,
            identity.signature(),
            identity.issuer_pk(),
            &[IdentityField::IsMajor],
        )
        .unwrap();

        let revealed =
            Verifier::verify_signature_pok(&proof.proof_request, &proof.signature, &proof.nonce)
                .unwrap();

        assert_eq!(revealed.len(), 1);
        let expected_field = identity.get_field(&IdentityField::IsMajor).unwrap();
        assert_eq!(revealed[0], SignatureMessage::hash(&expected_field));
    }

    #[test]
    fn test_verification_fails_with_tampered_nonce() {
        let eve = create_identity();

        let proof = generate_selective_disclosure(
            &eve,
            eve.signature(),
            eve.issuer_pk(),
            &[IdentityField::Name],
        )
        .unwrap();

        let tampered_nonce = Verifier::generate_proof_nonce();
        let result =
            Verifier::verify_signature_pok(&proof.proof_request, &proof.signature, &tampered_nonce);
        assert!(result.is_err());
    }
}
