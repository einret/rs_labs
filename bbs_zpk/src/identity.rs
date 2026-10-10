use crate::error::CompressedSignature;
use crate::traits::{FieldExtractor, FullExtractor};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use bbs::prelude::PublicKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityField {
    Name,
    IsMajor,
    Address,
}

impl IdentityField {
    const ORDER: &'static [IdentityField] = &[
        IdentityField::Name,
        IdentityField::IsMajor,
        IdentityField::Address,
    ];
    fn descriptor(self) -> FieldDescriptor {
        match self {
            Self::Name => FieldDescriptor {
                get_bytes: |body| body.name.as_bytes().to_vec(),
            },
            Self::IsMajor => FieldDescriptor {
                get_bytes: |body| body.is_major.to_string().as_bytes().to_vec(),
            },
            Self::Address => FieldDescriptor {
                get_bytes: |body| body.address.as_bytes().to_vec(),
            },
        }
    }
}

/// The pure claim payload before or after issuance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityBody {
    pub name: String,
    pub is_major: bool,
    pub address: String,
}

struct FieldDescriptor {
    get_bytes: fn(&IdentityBody) -> Vec<u8>,
}

impl FullExtractor for IdentityBody {
    fn get_all_fields(&self) -> Vec<Vec<u8>> {
        IdentityField::ORDER
            .iter()
            .map(|f| (f.descriptor().get_bytes)(self))
            .collect()
    }
}

impl FieldExtractor<IdentityField> for IdentityBody {
    fn find_field_index(&self, field: &IdentityField) -> Option<usize> {
        IdentityField::ORDER.iter().position(|f| f == field)
    }

    fn get_field(&self, field: &IdentityField) -> Option<Vec<u8>> {
        // reuturn None if not found.
        self.find_field_index(field)?;
        Some((field.descriptor().get_bytes)(self))
    }
}

/// The issued credential: holds claims, the signature, and the issuer's public key.
#[derive(Clone, PartialEq, Eq)]
pub struct Identity {
    body: IdentityBody,
    signature: CompressedSignature,
    issuer_pk: PublicKey,
}

impl Identity {
    pub fn new(body: IdentityBody, signature: CompressedSignature, issuer_pk: PublicKey) -> Self {
        Self {
            body,
            signature,
            issuer_pk,
        }
    }

    pub fn body(&self) -> &IdentityBody {
        &self.body
    }

    pub fn signature(&self) -> &CompressedSignature {
        &self.signature
    }

    pub fn issuer_pk(&self) -> &PublicKey {
        &self.issuer_pk
    }
}

impl FullExtractor for Identity {
    fn get_all_fields(&self) -> Vec<Vec<u8>> {
        self.body.get_all_fields()
    }
}

impl FieldExtractor<IdentityField> for Identity {
    fn find_field_index(&self, field: &IdentityField) -> Option<usize> {
        self.body.find_field_index(field)
    }

    fn get_field(&self, field: &IdentityField) -> Option<Vec<u8>> {
        self.body.get_field(field)
    }
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base64_signature = STANDARD.encode(self.signature);
        f.debug_struct("Identity")
            .field("body", &self.body)
            .field("signature", &base64_signature)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_body_extraction() {
        let body = IdentityBody {
            name: "Bob".to_string(),
            is_major: false,
            address: "123 Elm St".to_string(),
        };

        assert_eq!(body.find_field_index(&IdentityField::Name), Some(0));
        assert_eq!(body.find_field_index(&IdentityField::IsMajor), Some(1));
        assert_eq!(body.find_field_index(&IdentityField::Address), Some(2));

        let fields = body.get_all_fields();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0], b"Bob");
        assert_eq!(fields[1], b"false");
        assert_eq!(fields[2], b"123 Elm St");
    }

    #[test]
    fn identity_accessors() {
        let (pk, _) = bbs::prelude::Issuer::new_keys(3).unwrap();
        let body = IdentityBody {
            name: "Alice".to_string(),
            is_major: true,
            address: "Wonderland".to_string(),
        };
        let signature = [42u8; 112];
        let identity = Identity::new(body.clone(), signature, pk.clone());

        assert_eq!(identity.body(), &body);
        assert_eq!(identity.body().name, "Alice");
        assert!(identity.body().is_major);
        assert_eq!(identity.signature(), &signature);
        assert_eq!(identity.issuer_pk(), &pk);
    }
}
