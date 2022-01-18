use crate::{
    backend::{ir::*, Backend, TypeInfo, TypeInfoTable},
    options::OptimizationSettings,
    parser::{
        ast::{BinaryExpressionType, SizeofType},
        Type, TypeNode,
    },
};

use super::jump_eval::JumpType;

pub struct EvaluationContext<'a> {
    pub vreg_counter: u32,
    pub label_counter: u32,
    pub variables: Vec<IRVariable>,
    pub strings: Vec<String>,
    pub unfixed_continue: Vec<(usize, u32)>,
    pub unfixed_break: Vec<(usize, u32)>,
    pub loop_depth: u32,
    pub struct_size_table: &'a Vec<TypeInfo>,
    pub struct_offset_table: &'a Vec<Vec<usize>>,
    pub backend: &'a dyn Backend,
    pub type_info: TypeInfoTable,

    pub optimization_settings: &'a OptimizationSettings,
}

impl<'a> EvaluationContext<'a> {
    pub fn next_vreg(&mut self) -> u32 {
        let vreg = self.vreg_counter;
        self.vreg_counter += 1;
        vreg
    }
    pub fn next_label(&mut self) -> u32 {
        let label = self.label_counter;
        self.label_counter += 1;
        label
    }
    pub fn add_string(&mut self, string: &String) -> u32 {
        let number = self.strings.len() as u32;
        self.strings.push(string.clone());
        number
    }
}

impl<'a> EvaluationContext<'a> {
    pub fn insert_place_holder_jump(&mut self, result: &mut Vec<IRInstruction>) -> (usize, u32) {
        let index = result.len();
        result.push(IRInstruction::Jmp(0));

        let label = self.insert_label(result);

        (index, label)
    }

    pub fn insert_place_holder_jump_phi(
        &mut self,
        result: &mut Vec<IRInstruction>,
        phi: Box<IRPhi>,
    ) -> (usize, u32) {
        let index = result.len();
        result.push(IRInstruction::Jmp(0));

        let label = self.insert_phi_label(result, phi);

        (index, label)
    }

    pub fn insert_fall_through(&mut self, result: &mut Vec<IRInstruction>) -> u32 {
        let label = self.label_counter;
        result.push(IRInstruction::Jmp(label));
        self.insert_label(result)
    }

    pub fn insert_label(&mut self, result: &mut Vec<IRInstruction>) -> u32 {
        let label = self.next_label();
        result.push(IRInstruction::Label(None, label));

        label
    }

    pub fn insert_phi_label(&mut self, result: &mut Vec<IRInstruction>, phi: Box<IRPhi>) -> u32 {
        let label = self.next_label();
        result.push(IRInstruction::Label(Some(phi), label));

        label
    }

    pub fn get_current_label(&self) -> u32 {
        let label = self.label_counter - 1;
        label
    }
}

impl<'a> EvaluationContext<'a> {
    pub fn enter_loop(&mut self) {
        self.loop_depth += 1;
    }

    pub fn add_break(&mut self, index: usize) {
        self.unfixed_break.push((index, self.loop_depth))
    }

    pub fn add_continue(&mut self, index: usize) {
        self.unfixed_continue.push((index, self.loop_depth))
    }

    pub fn fix_jumps(
        &mut self,
        result: &mut Vec<IRInstruction>,
        jumps: &[(usize, u32, IRSize, JumpType)],
        label: u32,
    ) {
        for &(index, vreg, size, jump_type) in jumps {
            result[index] = match jump_type {
                JumpType::Jcc => IRInstruction::Jcc(size, vreg, label),
                JumpType::Jnc => IRInstruction::Jnc(size, vreg, label),
            }
        }
    }

    pub fn fix_break_continue(
        &mut self,
        result: &mut Vec<IRInstruction>,
        break_label: u32,
        coninue_label: u32,
    ) {
        self.unfixed_break = self
            .unfixed_break
            .iter()
            .filter_map(|(i, depth)| {
                if *depth == self.loop_depth {
                    result[*i] = IRInstruction::Jmp(break_label);
                    None
                } else {
                    Some((*i, *depth))
                }
            })
            .collect();

        self.unfixed_continue = self
            .unfixed_continue
            .iter()
            .filter_map(|(i, depth)| {
                if *depth == self.loop_depth {
                    result[*i] = IRInstruction::Jmp(coninue_label);
                    None
                } else {
                    Some((*i, *depth))
                }
            })
            .collect();

        self.loop_depth -= 1;
    }
}

impl TypeInfoTable {
    pub fn get_irsize(&self, typ: &TypeNode, struct_info: &Vec<TypeInfo>) -> IRSize {
        use TypeNode::*;
        match typ {
            Char => self.char.irsize,
            Short => self.short.irsize,
            Int => self.int.irsize,
            Long => self.long.irsize,
            Pointer => IRSize::P,
            Struct(index) => IRSize::B(struct_info[*index].size as u16),
            Void => IRSize::V,
            _ => unreachable!(),
        }
    }

    fn get_sizeof(&self, typ: &TypeNode, struct_info: &Vec<TypeInfo>) -> u32 {
        use TypeNode::*;
        let res = match typ {
            Char => self.char.size,
            Short => self.short.size,
            Int => self.int.size,
            Long => self.long.size,
            Pointer => self.pointer.size,
            Struct(index) => struct_info[*index].size,
            Void => 1,
            _ => unreachable!(),
        };
        res as u32
    }

    fn get_size2(&self, typ: &Type, struct_info: &Vec<TypeInfo>) -> IRSize {
        self.get_irsize(&typ.nodes[0], struct_info)
    }

    pub fn eval_sizeof(&self, typ: &SizeofType, struct_info: &Vec<TypeInfo>) -> u32 {
        let ast_type = match typ {
            SizeofType::Type(_, typ) => typ,
            SizeofType::Expression(exp) => &exp.ast_type,
        };
        self.sizeof(ast_type, struct_info)
    }

    pub fn sizeof_element(&self, typ: &Type, struct_info: &Vec<TypeInfo>) -> u32 {
        if typ.is_array() {
            let (array_type, _) = typ.deconstruct();
            self.get_sizeof(&array_type, struct_info)
        } else {
            self.get_sizeof(&typ.nodes[0], struct_info)
        }
    }

    pub fn sizeof(&self, typ: &Type, struct_info: &Vec<TypeInfo>) -> u32 {
        if typ.is_array() {
            let (array_type, array_size) = typ.deconstruct();
            self.get_sizeof(&array_type, struct_info) * (array_size as u32)
        } else {
            self.get_sizeof(&typ.nodes[0], struct_info)
        }
    }

    pub fn int_ptr(&self, signed: bool) -> IRSize {
        assert!(signed); //Unsigned integers are currently unsupported
        match self.pointer.size {
            8 => IRSize::S64,
            4 => IRSize::S32,
            2 => IRSize::S16,
            _ => unreachable!(),
        }
    }

    pub fn size_t(&self) -> Type {
        vec![self.size_t.clone()].into()
    }
}

/*
impl<'a> (dyn Backend + 'a) {
    pub fn eval_sizeof(&self, typ: &SizeofType, struct_info: &Vec<TypeInfo>) -> u32 {
        let ast_type = match typ {
            SizeofType::Type(_, typ) => typ,
            SizeofType::Expression(exp) => &exp.ast_type,
        };
        self.sizeof(ast_type, struct_info)
    }

    fn get_size2(&self, typ: &Type, struct_info: &Vec<TypeInfo>) -> IRSize {
        self.get_size_struct(&typ.nodes[0], struct_info)
    }

    pub fn get_size_struct(&self, typ: &TypeNode, struct_info: &Vec<TypeInfo>) -> IRSize {
        if let TypeNode::Struct(index) = typ {
            IRSize::B(struct_info[*index].size as u16)
        } else {
            Backend::get_size(self, &typ)
        }
    }

    pub fn sizeof_element(&self, typ: &Type, struct_info: &Vec<TypeInfo>) -> u32 {
        if typ.is_array() {
            let (array_type, _) = typ.deconstruct();
            self.sizeof2(self.get_size_struct(&array_type, struct_info))
        } else {
            self.sizeof2(self.get_size2(&typ, struct_info))
        }
    }

    pub fn sizeof(&self, typ: &Type, struct_info: &Vec<TypeInfo>) -> u32 {
        if typ.is_array() {
            let (array_type, array_size) = typ.deconstruct();
            self.sizeof2(self.get_size_struct(&array_type, struct_info)) * (array_size as u32)
        } else {
            self.sizeof2(self.get_size2(&typ, struct_info))
        }
    }

    fn sizeof2(&self, size: IRSize) -> u32 {
        match size {
            IRSize::V => 1,
            IRSize::S8 => 1,
            IRSize::S16 => 2,
            IRSize::S32 => 4,
            IRSize::S64 => 8,
            IRSize::B(size) => size as u32,
            IRSize::P => self.sizeof_pointer(),
        }
    }
    pub fn int_ptr(&self, signed: bool) -> IRSize {
        assert!(signed); //Unsigned integers are currently unsupported
        match self.sizeof_pointer() {
            8 => IRSize::S64,
            4 => IRSize::S32,
            2 => IRSize::S16,
            _ => unreachable!(),
        }
    }

    pub fn size_t(&self) -> Type {
        vec![self.type].into()
    }
}*/

pub trait EvaluateSize {
    fn type_info<'a>(&'a self) -> &'a TypeInfoTable;
    fn struct_size_table<'a>(&'a self) -> &'a Vec<TypeInfo>;

    fn eval_sizeof(&self, typ: &SizeofType) -> u32 {
        self.type_info().eval_sizeof(typ, &self.struct_size_table())
    }
    fn get_size(&self, typ: &Type) -> IRSize {
        self.type_info().get_size2(typ, &self.struct_size_table())
    }
    fn sizeof(&self, typ: &Type) -> u32 {
        self.type_info().sizeof(typ, &self.struct_size_table())
    }
    fn sizeof_element(&self, typ: &Type) -> u32 {
        self.type_info()
            .sizeof_element(typ, &self.struct_size_table())
    }
    fn int_ptr(&self, signed: bool) -> IRSize {
        self.type_info().int_ptr(signed)
    }
    fn size_t(&self) -> Type {
        self.type_info().size_t()
    }
}

impl<'b> EvaluateSize for EvaluationContext<'b> {
    fn type_info(&self) -> &TypeInfoTable {
        &self.type_info
    }
    fn struct_size_table<'a>(&'a self) -> &'a Vec<TypeInfo> {
        &self.struct_size_table
    }
}

impl<'a> EvaluationContext<'a> {
    pub fn promote(
        &mut self,
        result: &mut Vec<IRInstruction>,
        size: IRSize,
        from: IRSize,
        vreg: u32,
    ) -> u32 {
        if size == from {
            return vreg;
        }
        let temp = self.next_vreg();
        result.push(IRInstruction::Cvs(size, temp, from, vreg));
        temp
    }
}

impl BinaryExpressionType {
    pub fn get_size(&self, context: &mut EvaluationContext, left: &Type, _right: &Type) -> IRSize {
        use BinaryExpressionType::*;
        let size = context.get_size(left);
        match self {
            Subtract => size,
            Add => size,
            Multiply | Divide | BinOr | BinAnd | Equal | Inequal | Less | LessEqual | Greater
            | GreaterEqual => size,
            _ => unreachable!(),
        }
    }
}
