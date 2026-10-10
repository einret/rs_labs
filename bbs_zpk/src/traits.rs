pub trait FullExtractor {
    fn get_all_fields(&self) -> Vec<Vec<u8>>;
}

pub trait FieldExtractor<F> {
    fn find_field_index(&self, field: &F) -> Option<usize>;
    fn get_field(&self, field: &F) -> Option<Vec<u8>>;
}
