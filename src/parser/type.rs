#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Name(String),
    Function(Vec<Vec<Type>>),
}
