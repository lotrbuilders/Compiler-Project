use super::RegisterInterface;

use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::{Index, Not};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterClass<R: RegisterInterface> {
    pub phantom: PhantomData<*const R>,
    pub registers: &'static [bool],
}

#[allow(dead_code)]
impl<R: RegisterInterface> RegisterClass<R> {
    pub fn is_target(&self) -> Option<R> {
        let mut count = 0;
        let mut result = R::REG_DEFAULT;
        for reg in 0..self.registers.len() {
            if self.registers[reg] {
                result = R::REG_LOOKUP[reg];
                count += 1;
            }
            if count > 1 {
                return None;
            }
        }
        return Some(result);
    }
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

impl<R: RegisterInterface> Index<usize> for RegisterClass<R> {
    type Output = bool;
    fn index(&self, index: usize) -> &Self::Output {
        &self.registers[index]
    }
}

impl<R: RegisterInterface> Index<R> for RegisterClass<R> {
    type Output = bool;
    fn index(&self, index: R) -> &Self::Output {
        &self[<R as Into<usize>>::into(index)]
    }
}

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
            if self.class.registers[self.index] {
                self.index += 1;
                return result;
            }
            self.index += 1;
        }
        None
    }
}

impl<R: RegisterInterface> Not for RegisterClass<R> {
    type Output = RegisterClassNot<R>;

    fn not(self) -> Self::Output {
        RegisterClassNot { class: self }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterClassNot<R: RegisterInterface> {
    class: RegisterClass<R>,
}

impl<R: RegisterInterface> IntoIterator for RegisterClassNot<R> {
    type Item = R;
    type IntoIter = RegisterClassNotIter<R>;

    fn into_iter(self) -> Self::IntoIter {
        RegisterClassNotIter {
            class: self.class,
            index: 0,
        }
    }
}

pub struct RegisterClassNotIter<R: RegisterInterface> {
    class: RegisterClass<R>,
    index: usize,
}

impl<R: RegisterInterface> Iterator for RegisterClassNotIter<R> {
    type Item = R;
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < R::REG_COUNT {
            let result = Some(R::REG_LOOKUP[self.index].clone());
            if !self.class.registers[self.index] {
                self.index += 1;
                return result;
            }
            self.index += 1;
        }
        None
    }
}
