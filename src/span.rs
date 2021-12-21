use std::fmt::Display;

/// Struct too show were a token or AST section originates from.
// .file_index gives the index into the list of files
// .line is the line number of the start of the token/AST.
// .column is the column number of the start.
// .offset is the character offset at which the area starts.
// .length is the length offset at which the area starts.
#[allow(dead_code)]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Span {
    file_index: u32,
    line: u32,
    column: u32,
    offset: u32,
    length: u32,
}

#[allow(dead_code)]
impl Span {
    pub fn new(file_index: u32, line: u32, column: u32, offset: u32, length: u32) -> Self {
        Span {
            file_index,
            line,
            column,
            offset,
            length,
        }
    }
    pub fn line(&self) -> u32 {
        self.line
    }
    pub fn column(&self) -> u32 {
        self.column
    }
    pub fn offset(&self) -> u32 {
        self.offset
    }
    pub fn length(&self) -> u32 {
        self.length
    }

    /// Transform the section from Span Self to Span other too the combined span
    pub fn to(&self, other: &Span) -> Span {
        // Ensure in debug mode that the self comes before other
        if cfg!(debug_assertions) && self.offset > other.offset {
            log::warn!(
                "Span::to is given in the wrong order: (self:{:?} - other:{:?}",
                self,
                other
            );
        }
        Span::new(
            self.file_index,
            self.line,
            self.column,
            self.offset,
            other.offset - self.offset + other.length,
        )
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::file_table;
        let file = file_table::get_sourcefile(self.file_index);
        write!(f, "{}:{}:{}:", file, self.line, self.column)
    }
}
