use super::RegisterInterface;

use std::fmt::Display;
//use std::iter::FromIterator;
use std::marker::PhantomData;
//use std::ops::BitAnd;
//use std::ops::BitOr;
use std::ops::Index;
//use std::ops::Sub;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterClass<R: RegisterInterface> {
    pub phantom: PhantomData<*const R>,
    pub registers: &'static [bool],
}

#[allow(dead_code)]
impl<R: RegisterInterface> RegisterClass<R> {
    pub fn iter<'a>(&'a self) -> RegisterClassIterRef<'a, R> {
        self.into_iter()
    }
    pub(super) fn add(&mut self, reg: R) {
        let _ = reg;
        unreachable!();
    }
}

impl<'a, R: RegisterInterface> IntoIterator for &'a RegisterClass<R> {
    type Item = &'a R;
    type IntoIter = RegisterClassIterRef<'a, R>;

    fn into_iter(self) -> Self::IntoIter {
        RegisterClassIterRef {
            class: &self,
            index: 0,
        }
    }
}

impl<R: RegisterInterface> IntoIterator for RegisterClass<R> {
    type Item = R;
    type IntoIter = RegisterClassIter<R>;

    fn into_iter(self) -> Self::IntoIter {
        RegisterClassIter {
            class: self,
            index: 0,
        }
    }
}

/*
impl<R: RegisterInterface> FromIterator<R> for RegisterClass {
    fn from_iter<T: IntoIterator<Item = Register>>(iter: T) -> Self {
        let mut registers = [false; REG_COUNT];
        for reg in iter {
            registers[reg as usize] = true;
        }
        RegisterClass { registers }
    }
}

impl<'a, R: RegisterInterface> FromIterator<&'a Register> for RegisterClass<R> {
    fn from_iter<T: IntoIterator<Item = &'a Register>>(iter: T) -> Self {
        let mut registers = [false; REG_COUNT];
        for reg in iter {
            registers[*reg as usize] = true;
        }
        RegisterClass { registers }
    }
}
*/

impl<R: RegisterInterface> Index<usize> for RegisterClass<R> {
    type Output = bool;
    fn index(&self, index: usize) -> &Self::Output {
        &self.registers[index]
    }
}

impl<R: RegisterInterface> Index<R> for RegisterClass<R> {
    type Output = bool;
    fn index(&self, index: R) -> &Self::Output {
        &self[index.into()]
    }
}

/*
impl<R: RegisterInterface> BitOr<&Self> for RegisterClass<R> {
    type Output = RegisterClass<R>;
    fn bitor(self, rhs: &Self) -> Self::Output {
        &self | rhs
    }
}

impl<R: RegisterInterface> BitOr for RegisterClass<R> {
    type Output = RegisterClass<R>;
    fn bitor(self, rhs: Self) -> Self::Output {
        &self | &rhs
    }
}

/// Or represents the union
impl<R: RegisterInterface> BitOr for &RegisterClass<R> {
    type Output = RegisterClass<R>;
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

impl<R: RegisterInterface> BitAnd<&Self> for RegisterClass {
    type Output = RegisterClass;
    fn bitand(self, rhs: &Self) -> Self::Output {
        &self & rhs
    }
}

impl<R: RegisterInterface> BitAnd for RegisterClass {
    type Output = RegisterClass;
    fn bitand(self, rhs: Self) -> Self::Output {
        &self & &rhs
    }
}

/// And represents the intersection
impl<R: RegisterInterface> BitAnd for &RegisterClass {
    type Output = RegisterClass;
    fn bitand(self, rhs: Self) -> Self::Output {
        (0..R::REG_COUNT)
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

impl<R: RegisterInterface> Sub<&Self> for RegisterClass {
    type Output = RegisterClass;
    fn sub(self, rhs: &Self) -> Self::Output {
        &self - rhs
    }
}

impl<R: RegisterInterface> Sub for RegisterClass {
    type Output = RegisterClass;
    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}

/// Subtract represents the complement
impl<R: RegisterInterface> Sub for &RegisterClass {
    type Output = RegisterClass;
    fn sub(self, rhs: Self) -> Self::Output {
        (0..R::REG_COUNT)
            .filter_map(|i| {
                if self.registers[i] && !rhs.registers[i] {
                    Some(REG_LOOKUP[i])
                } else {
                    None
                }
            })
            .collect()
    }
}*/

impl<R: RegisterInterface + Display> Display for RegisterClass<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for reg in self.iter() {
            write!(f, "{},", reg)?;
        }
        write!(f, "]")
    }
}

#[derive(Clone)]
pub struct RegisterClassIterRef<'a, R: RegisterInterface> {
    class: &'a RegisterClass<R>,
    index: usize,
}
impl<'a, R: RegisterInterface> Iterator for RegisterClassIterRef<'a, R> {
    type Item = &'a R;
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < R::REG_COUNT {
            if self.class.registers[self.index] {
                let result = Some(&R::REG_LOOKUP[self.index]);
                self.index += 1;
                return result;
            }
            self.index += 1;
        }
        None
    }
}

pub struct RegisterClassIter<R: RegisterInterface> {
    class: RegisterClass<R>,
    index: usize,
}
impl<R: RegisterInterface> Iterator for RegisterClassIter<R> {
    type Item = R;
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < R::REG_COUNT {
            let result = Some(R::REG_LOOKUP[self.index].clone());
            self.index += 1;
            if self.class.registers[self.index] {
                return result;
            }
        }
        None
    }
}
