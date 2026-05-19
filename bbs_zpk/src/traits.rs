use crate::error::Result;
use bbs::prelude::PublicKey;

pub trait FullExtractor {
    fn get_all_fields(&self) -> Vec<Vec<u8>>;
}

pub trait FieldExtractor<F> {
    fn find_field_index(&self, field: &F) -> Option<usize>;
    fn get_field(&self, field: &F) -> Option<Vec<u8>>;
}

pub trait Signer: Send + Sync {
    fn public_key(&self) -> &PublicKey;
    fn sign(&self, entity: &dyn FullExtractor) -> Result<[u8; 112]>;
}
