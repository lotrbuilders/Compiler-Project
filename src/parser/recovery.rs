// The RecoveryStrategy is used to specify how the parser might recover from an error
#[allow(dead_code)]

/* UpTo         - Remove all tokens up to the first occurance of this character
** Until        - Remove all tokens until and including the first occurance of this character
** UntilBrace   - Remove all all tokens until the first occurance of this character and the entire braced block,
**                Takes into account possible sub-blocks.
**                Allows specifying the end token, which parses as if the block has already started
** Or           - Use either of the two recovery strategies(possibly recursive)
** Nothing      - Do not actively recover(For use by expect)
*/
pub enum RecoveryStrategy {
    UpTo(char),
    Until(char),
    UntilBraced(char),
    Or(Box<RecoveryStrategy>, Box<RecoveryStrategy>),
    Nothing,
}

#[allow(dead_code)]
impl RecoveryStrategy {
    pub fn or(a: RecoveryStrategy, b: RecoveryStrategy) -> RecoveryStrategy {
        RecoveryStrategy::Or(Box::new(a), Box::new(b))
    }
}
