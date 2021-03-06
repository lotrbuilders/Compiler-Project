use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::backend::{
    ir::IRInstruction,
    register_allocation::{RegisterAllocation, RegisterClass, RegisterRelocation, RegisterUse},
};
use crate::ir::ir::IRSize;

pub trait RegisterInterface
where
    Self: Sized
        + 'static
        + Into<usize>
        + Into<u32>
        + PartialEq
        + Display
        + Debug
        + Clone
        + Copy
        + Hash
        + Eq,
{
    const REG_COUNT: usize;
    const REG_LOOKUP: &'static [Self];
    const REG_DEFAULT: Self;
    const REG_DEFAULT_CLASS: RegisterClass<Self>;
    const CALL_REGS: &'static [RegisterClass<Self>];
}

pub trait RegisterBackend {
    type RegisterType: RegisterInterface;
    fn is_instruction(&self, rule: u16) -> bool;
    fn is_two_address(&self, rule: u16) -> bool;
    fn is_jump(&self, index: usize) -> bool;

    fn get_clobbered(&self, index: u32) -> Vec<Self::RegisterType>;
    fn get_clobbered_after(&self, index: u32) -> Vec<Self::RegisterType>;
    fn find_uses(&mut self) -> RegisterUse<Self::RegisterType>;
    fn get_instructions<'a>(&'a self) -> &Vec<IRInstruction>;
    fn get_rule(&self, index: usize) -> u16;
    fn get_arguments<'a>(&'a self) -> &'a Vec<Option<u32>>;
    fn get_function_length(&self) -> usize;
    fn get_vreg_count(&self) -> u32;
    fn get_vreg_size(&self, vreg: u32) -> IRSize;

    fn simple_get_spot(&self, vreg: u32) -> u32;
    fn simple_adjust_stack_size(&mut self, vreg: i32);

    fn set_used_registers(&mut self, used_registers: Vec<bool>);
    fn set_allocation(&mut self, allocation: Vec<RegisterAllocation<Self::RegisterType>>);
    fn set_reg_relocations(
        &mut self,
        reg_relocations: Vec<Vec<RegisterRelocation<Self::RegisterType>>>,
    );

    fn get_vregisters(
        &self,
        index: u32,
        rule: u16,
    ) -> (
        smallvec::SmallVec<[(u32, RegisterClass<Self::RegisterType>); 4]>,
        Option<(u32, RegisterClass<Self::RegisterType>)>,
    );
}
