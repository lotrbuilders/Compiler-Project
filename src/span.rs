use log::warn;

/// Struct too show were a token or AST section originates from
/// .line is the line number of the start of the token/AST
/// .column is the column number of the start
/// .offset is the character offset at which the area starts
/// .length is the length offset at which the area starts
#[allow(dead_code)]
#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct Span {
    line: i32,
    column: i32,
    offset: i32,
    length: i32,
}

#[allow(dead_code)]
impl Span {
    pub fn new(line: i32, column: i32, offset: i32, length: i32) -> Self {
        Span {
            line,
            column,
            offset,
            length,
        }
    }
    pub fn line(&self) -> i32 {
        self.line
    }
    pub fn column(&self) -> i32 {
        self.column
    }
    pub fn offset(&self) -> i32 {
        self.offset
    }
    pub fn length(&self) -> i32 {
        self.length
    }

    /// Transform the section from Span Self to Span other too the combined span
    pub fn to(&self, other: &Span) -> Span {
        // Ensure in debug mode that the self comes before other
        if cfg!(debug_assertions) && self.offset > other.offset {
            warn!(
                "Span::to is given in the wrong order: (self:{:?} - other:{:?}",
                self, other
            );
        }
        Span::new(
            self.line,
            self.column,
            self.offset,
            other.offset - self.offset + other.length,
        )
    }
}
