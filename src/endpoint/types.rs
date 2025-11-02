#[derive(Clone)]
pub struct Page {
    pub mime: String,
    pub content: Vec<u8>,
    pub status: u16,
}
