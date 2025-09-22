#[derive(Debug, Clone)]
pub struct Edit {
    pub range: (usize, usize), // start_byte, end_byte
    pub content: String,
}


pub struct EditTarget<T> {
    pub range: (usize, usize),
    pub items: Vec<T>,
}
