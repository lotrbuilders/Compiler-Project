pub enum RecoveryStrategy {
    UpTo(char),
    Until(char),
    UntilBraced(char),
    Or(Box<RecoveryStrategy>, Box<RecoveryStrategy>),
    Nothing,
}

impl RecoveryStrategy {
    pub fn or(a: RecoveryStrategy, b: RecoveryStrategy) -> RecoveryStrategy {
        RecoveryStrategy::Or(Box::new(a), Box::new(b))
    }
}
