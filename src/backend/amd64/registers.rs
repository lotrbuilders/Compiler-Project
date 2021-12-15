use std::fmt::Display;
use std::iter::FromIterator;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::Index;
use std::ops::Sub;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterClass {
    pub registers: [bool; REG_COUNT],
}

#[allow(dead_code)]
impl RegisterClass {
    pub(super) fn new(registers: [bool; REG_COUNT]) -> RegisterClass {
        RegisterClass { registers }
    }
    pub(super) fn iter<'a>(&'a self) -> RegisterClassIterRef<'a> {
        self.into_iter()
    }
    pub(super) fn add(&mut self, reg: Register) {
        self.registers[reg as usize] = true;
    }
}

impl<'a> IntoIterator for &'a RegisterClass {
    type Item = Register;
    type IntoIter = RegisterClassIterRef<'a>;

    fn into_iter(self) -> Self::IntoIter {
        RegisterClassIterRef {
            class: &self,
            index: 0,
        }
    }
}

impl IntoIterator for RegisterClass {
    type Item = Register;
    type IntoIter = RegisterClassIter;

    fn into_iter(self) -> Self::IntoIter {
        RegisterClassIter {
            class: self,
            index: 0,
        }
    }
}

impl FromIterator<Register> for RegisterClass {
    fn from_iter<T: IntoIterator<Item = Register>>(iter: T) -> Self {
        let mut registers = [false; REG_COUNT];
        for reg in iter {
            registers[reg as usize] = true;
        }
        RegisterClass { registers }
    }
}

impl<'a> FromIterator<&'a Register> for RegisterClass {
    fn from_iter<T: IntoIterator<Item = &'a Register>>(iter: T) -> Self {
        let mut registers = [false; REG_COUNT];
        for reg in iter {
            registers[*reg as usize] = true;
        }
        RegisterClass { registers }
    }
}

impl Index<usize> for RegisterClass {
    type Output = bool;
    fn index(&self, index: usize) -> &Self::Output {
        &self.registers[index]
    }
}

impl Index<Register> for RegisterClass {
    type Output = bool;
    fn index(&self, index: Register) -> &Self::Output {
        &self[index as usize]
    }
}

impl BitOr<&Self> for RegisterClass {
    type Output = RegisterClass;
    fn bitor(self, rhs: &Self) -> Self::Output {
        &self | rhs
    }
}

impl BitOr for RegisterClass {
    type Output = RegisterClass;
    fn bitor(self, rhs: Self) -> Self::Output {
        &self | &rhs
    }
}

/// Or represents the union
impl BitOr for &RegisterClass {
    type Output = RegisterClass;
    fn bitor(self, rhs: Self) -> Self::Output {
        (0..REG_COUNT)
            .filter_map(|i| {
                if self.registers[i] || rhs.registers[i] {
                    Some(REG_LOOKUP[i])
                } else {
                    None
                }
            })
            .collect()
    }
}

impl BitAnd<&Self> for RegisterClass {
    type Output = RegisterClass;
    fn bitand(self, rhs: &Self) -> Self::Output {
        &self & rhs
    }
}

impl BitAnd for RegisterClass {
    type Output = RegisterClass;
    fn bitand(self, rhs: Self) -> Self::Output {
        &self & &rhs
    }
}

/// And represents the intersection
impl BitAnd for &RegisterClass {
    type Output = RegisterClass;
    fn bitand(self, rhs: Self) -> Self::Output {
        (0..REG_COUNT)
            .filter_map(|i| {
                if self.registers[i] && rhs.registers[i] {
                    Some(REG_LOOKUP[i])
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Sub<&Self> for RegisterClass {
    type Output = RegisterClass;
    fn sub(self, rhs: &Self) -> Self::Output {
        &self - rhs
    }
}

impl Sub for RegisterClass {
    type Output = RegisterClass;
    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}

/// Subtract represents the complement
impl Sub for &RegisterClass {
    type Output = RegisterClass;
    fn sub(self, rhs: Self) -> Self::Output {
        (0..REG_COUNT)
            .filter_map(|i| {
                if self.registers[i] && !rhs.registers[i] {
                    Some(REG_LOOKUP[i])
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Display for RegisterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for reg in self.iter() {
            write!(f, "{},", reg)?;
        }
        write!(f, "]")
    }
}

#[derive(Clone)]
pub struct RegisterClassIterRef<'a> {
    class: &'a RegisterClass,
    index: usize,
}
impl Iterator for RegisterClassIterRef<'_> {
    type Item = Register;
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < REG_COUNT {
            if self.class.registers[self.index] {
                let result = Some(REG_LOOKUP[self.index]);
                self.index += 1;
                return result;
            }
            self.index += 1;
        }
        None
    }
}

pub struct RegisterClassIter {
    class: RegisterClass,
    index: usize,
}
impl Iterator for RegisterClassIter {
    type Item = Register;
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < REG_COUNT {
            let result = Some(REG_LOOKUP[self.index]);
            self.index += 1;
            if self.class.registers[self.index] {
                return result;
            }
        }
        None
    }
}
