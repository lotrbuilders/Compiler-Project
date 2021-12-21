use super::super::registers::*;
use std::fmt::Display;
use std::iter::FromIterator;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::Index;
use std::ops::Sub;

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
