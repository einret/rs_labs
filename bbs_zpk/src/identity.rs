use crate::traits::{FieldExtractor, FullExtractor, Signer};
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityField {
    Name,
    IsMajor,
    Address,
}

impl std::fmt::Display for IdentityField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityField::Name => write!(f, "Name"),
            IdentityField::IsMajor => write!(f, "IsMajor"),
            IdentityField::Address => write!(f, "Address"),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Identity {
    name: String,
    is_major: bool,
    address: String,
    signature: [u8; 112],
    signer: Arc<dyn Signer + Send + Sync>,
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base64_signature = STANDARD.encode(self.signature);
        f.debug_struct("Identity")
            .field("name", &self.name)
            .field("is_major", &self.is_major)
            .field("address", &self.address)
            .field("signature", &base64_signature)
            .finish()
    }
}

// Un descripteur par champ : un nom (pour debug) et une fonction d'extraction
struct FieldDescriptor<'a> {
    name: &'a str,
    field: IdentityField,
    get_bytes: fn(&Identity) -> Vec<u8>,
}

const FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        name: "name",
        field: IdentityField::Name,
        get_bytes: |id| id.name.as_bytes().to_vec(),
    },
    FieldDescriptor {
        name: "is_major",
        field: IdentityField::IsMajor,
        get_bytes: |id| id.is_major.to_string().as_bytes().to_vec(),
    },
    FieldDescriptor {
        name: "address",
        field: IdentityField::Address,
        get_bytes: |id| id.address.as_bytes().to_vec(),
    },
];

impl Identity {
    pub fn new(
        name: String,
        is_major: bool,
        address: String,
        signer: Arc<dyn Signer + Send + Sync>,
    ) -> Result<Self> {
        let mut identity = Self {
            name,
            is_major,
            address,
            signature: [0; 112],
            signer,
        };
        identity.signature = identity.signer.sign(&identity)?;
        Ok(identity)
    }
    pub(crate) fn signature(&self) -> &[u8; 112] {
        &self.signature
    }
    pub(crate) fn signer(&self) -> Arc<dyn Signer + Send + Sync> {
        self.signer.clone()
    }
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

impl FullExtractor for Identity {
    fn get_all_fields(&self) -> Vec<Vec<u8>> {
        FIELDS.iter().map(|fd| (fd.get_bytes)(self)).collect()
    }
}
impl FieldExtractor<IdentityField> for Identity {
    fn find_field_index(&self, field: &IdentityField) -> Option<usize> {
        FIELDS.iter().position(|fd| fd.field == *field)
    }

    fn get_field(&self, field: &IdentityField) -> Option<Vec<u8>> {
        FIELDS
            .iter()
            .find(|fd| fd.field == *field)
            .map(|fd| (fd.get_bytes)(self))
    }
}
