use crate::error::{AppError, Result};
use crate::traits::{FullExtractor, Signer};
use bbs::prelude::*;

const DEFAULT_MESSAGE_COUNT: usize = 5;

pub(crate) struct MyIssuer {
    sk: SecretKey,
    pk: PublicKey,
    messages_len: usize,
}

impl MyIssuer {
    pub(crate) fn new(n: usize) -> Result<Self> {
        let (pk, sk) = Issuer::new_keys(n)?;
        Ok(Self {
            sk,
            pk,
            messages_len: n,
        })
    }
}

impl Signer for MyIssuer {
    fn public_key(&self) -> &PublicKey {
        &self.pk
    }
    fn sign(&self, entity: &dyn FullExtractor) -> Result<[u8; 112]> {
        let messages = entity.get_all_fields();
        if messages.len() != self.messages_len {
            return Err(AppError::SigningError {
                entity_description: "generic entity".to_string(),
                reason: "Invalid message count".to_string(),
                inner: None,
            });
        }
        let messages_to_sign: Vec<SignatureMessage> =
            messages.iter().map(|m| SignatureMessage::hash(m)).collect();

        let signature = Signature::new(messages_to_sign.as_slice(), &self.sk, &self.pk)?;

        Ok(signature.to_bytes_compressed_form())
    }
}

impl Default for MyIssuer {
    fn default() -> Self {
        let (pk, sk) = Issuer::new_keys(DEFAULT_MESSAGE_COUNT)
            .expect("Échec critique de l'initialisation par défaut de MyIssuer");
        Self {
            sk,
            pk,
            messages_len: DEFAULT_MESSAGE_COUNT,
        }
    }
}
