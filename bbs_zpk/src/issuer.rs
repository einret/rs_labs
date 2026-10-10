use crate::error::{AppError, CompressedSignature, Result};
use crate::identity::{Identity, IdentityBody};
use crate::traits::FullExtractor;
use bbs::prelude::*;
use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
pub(crate) struct MyIssuer {
    sk: SecretKey,
    #[zeroize(skip)]
    pk: PublicKey,
}

impl MyIssuer {
    pub(crate) fn new(n: usize) -> Result<Self> {
        let (pk, sk) = Issuer::new_keys(n)?;
        Ok(Self { sk, pk })
    }

    #[allow(dead_code)]
    pub(crate) fn public_key(&self) -> &PublicKey {
        &self.pk
    }

    fn sign(&self, entity: &dyn FullExtractor) -> Result<CompressedSignature> {
        let messages = entity.get_all_fields();
        let expected_count = self.pk.message_count();
        if messages.len() != expected_count {
            return Err(AppError::SigningError {
                entity_description: "generic entity".to_string(),
                reason: format!(
                    "expected {} messages, got {}",
                    expected_count,
                    messages.len()
                ),
                inner: None,
            });
        }
        let messages_to_sign: Vec<SignatureMessage> =
            messages.iter().map(SignatureMessage::hash).collect();

        let signature =
            Signature::new(messages_to_sign.as_slice(), &self.sk, &self.pk).map_err(|err| {
                AppError::SigningError {
                    entity_description: "generic entity".to_string(),
                    reason: "failed to compute BBS+ signature".to_string(),
                    inner: Some(err),
                }
            })?;

        Ok(signature.to_bytes_compressed_form())
    }
    pub(crate) fn sign_identity(&self, body: &IdentityBody) -> Result<crate::identity::Identity> {
        let signature = self.sign(body)?;
        Ok(Identity::new(body.clone(), signature, self.pk.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issuer_rejects_mismatched_message_count() {
        let issuer = MyIssuer::new(5).unwrap(); // expects 5
        let body = IdentityBody {
            name: "Dave".to_string(),
            is_major: false,
            address: "Nowhere".to_string(),
        };
        let res = issuer.sign_identity(&body);
        assert!(res.is_err());
    }
}
