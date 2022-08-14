//! Data exchange API (clipboard & drag/drop)

#[derive(Clone, Debug)]
pub struct TypedData {
    pub type_id: &'static str,
    pub data: Vec<u8>,
}
