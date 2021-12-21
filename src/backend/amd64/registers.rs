use super::register_allocation::RegisterClass;
use std::fmt::Display;

// Registers classes that are used. Should be automatically generated
pub const REG_COUNT: usize = 14;
#[allow(dead_code)]

pub const REG_CLASS_EAX: RegisterClass = RegisterClass {
    registers: [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false,
    ],
};
pub const REG_CLASS_EDI: RegisterClass = RegisterClass {
    registers: [
        false, false, false, true, false, false, false, false, false, false, false, false, false,
        false,
    ],
};
pub const REG_CLASS_ESI: RegisterClass = RegisterClass {
    registers: [
        false, false, false, false, true, false, false, false, false, false, false, false, false,
        false,
    ],
};
pub const REG_CLASS_ECX: RegisterClass = RegisterClass {
    registers: [
        false, true, false, false, false, false, false, false, false, false, false, false, false,
        false,
    ],
};
pub const REG_CLASS_EDX: RegisterClass = RegisterClass {
    registers: [
        false, false, true, false, false, false, false, false, false, false, false, false, false,
        false,
    ],
};
pub const REG_CLASS_R8: RegisterClass = RegisterClass {
    registers: [
        false, false, false, false, false, true, false, false, false, false, false, false, false,
        false,
    ],
};
pub const REG_CLASS_R9: RegisterClass = RegisterClass {
    registers: [
        false, false, false, false, false, false, true, false, false, false, false, false, false,
        false,
    ],
};

pub const CALL_REGS: [&'static RegisterClass; 6] = [
    &REG_CLASS_EDI,
    &REG_CLASS_ESI,
    &REG_CLASS_ECX,
    &REG_CLASS_EDX,
    &REG_CLASS_R8,
    &REG_CLASS_R9,
];
pub const REG_CLASS_IREG: RegisterClass = RegisterClass {
    registers: [true; REG_COUNT],
};
pub const REG_CLASS_EMPTY: RegisterClass = RegisterClass {
    registers: [false; REG_COUNT],
};

pub const REG_LOOKUP: [Register; REG_COUNT] = {
    use Register::*;
    [
        Rax, Rcx, Rdx, Rdi, Rsi, R8, R9, R10, R11, Rbx, R12, R13, R14, R15,
    ]
};

// Currently only caller safed registers
// An enum for all available registers to show effects
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Register {
    Rax = 0,
    Rcx = 1,
    Rdx = 2,
    Rdi = 3,
    Rsi = 4,
    R8 = 5,
    R9 = 6,
    R10 = 7,
    R11 = 8,
    Rbx = 9,
    R12 = 10,
    R13 = 11,
    R14 = 12,
    R15 = 13,
}
impl Register {
    pub fn to_string(&self) -> &'static str {
        self.to_string_i32()
    }

    pub fn to_string_i64(&self) -> &'static str {
        match self {
            Self::Rax => "rax",
            Self::Rcx => "rcx",
            Self::Rdx => "rdx",
            Self::Rdi => "rdi",
            Self::Rsi => "rsi",
            Self::R8 => "r8",
            Self::R9 => "r9",
            Self::R10 => "r10",
            Self::R11 => "r11",
            Self::Rbx => "rbx",
            Self::R12 => "r12",
            Self::R13 => "r13",
            Self::R14 => "r14",
            Self::R15 => "r15",
        }
    }

    pub fn to_string_i32(&self) -> &'static str {
        match self {
            Self::Rax => "eax",
            Self::Rcx => "ecx",
            Self::Rdx => "edx",
            Self::Rdi => "edi",
            Self::Rsi => "esi",
            Self::R8 => "r8d",
            Self::R9 => "r9d",
            Self::R10 => "r10d",
            Self::R11 => "r11d",
            Self::Rbx => "ebx",
            Self::R12 => "r12d",
            Self::R13 => "r13d",
            Self::R14 => "r14d",
            Self::R15 => "r15d",
        }
    }

    pub fn to_string_i8(&self) -> &'static str {
        match self {
            Self::Rax => "al",
            Self::Rcx => "cl",
            Self::Rdx => "dl",
            Self::Rdi => "dil",
            Self::Rsi => "sil",
            Self::R8 => "r8l",
            Self::R9 => "r9l",
            Self::R10 => "r10l",
            Self::R11 => "r11d",
            Self::Rbx => "bl",
            Self::R12 => "r12l",
            Self::R13 => "r13l",
            Self::R14 => "r14l",
            Self::R15 => "r15l",
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match f.precision() {
            Some(8) => write!(f, "{}", self.to_string_i8())?,
            Some(32) => write!(f, "{}", self.to_string_i32())?,
            Some(64) => write!(f, "{}", self.to_string_i64())?,
            Some(s) => log::error!("Unsupported precision {}", s),
            None => write!(f, "{}", self.to_string())?,
        }
        Ok(())
    }
}
