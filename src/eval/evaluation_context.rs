use crate::{
    backend::{ir::*, Backend, TypeInfo},
    parser::{
        ast::{BinaryExpressionType, SizeofType},
        Type,
    },
};

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

impl<'a> (dyn Backend + 'a) {
    pub fn eval_sizeof(&self, typ: &SizeofType) -> u32 {
        let ast_type = match typ {
            SizeofType::Type(typ) => typ,
            SizeofType::Expression(exp) => &exp.ast_type,
        };
        self.sizeof(ast_type)
    }

    fn get_size2(&self, typ: &Type) -> IRSize {
        Backend::get_size(self, &typ.nodes[0])
    }

    pub fn sizeof_element(&self, typ: &Type) -> u32 {
        if typ.is_array() {
            let (array_type, _) = typ.deconstruct();
            self.sizeof2(self.get_size(&array_type))
        } else {
            self.sizeof2(self.get_size2(&typ))
        }
    }

    pub fn sizeof(&self, typ: &Type) -> u32 {
        if typ.is_array() {
            let (array_type, array_size) = typ.deconstruct();
            self.sizeof2(self.get_size(&array_type)) * (array_size as u32)
        } else {
            self.sizeof2(self.get_size2(&typ))
        }
    }

    fn sizeof2(&self, size: IRSize) -> u32 {
        match size {
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
        vec![self.typeof_size_t()].into()
    }
}

impl<'a> EvaluationContext<'a> {
    pub fn get_size(&'a self, typ: &Type) -> IRSize {
        self.backend.get_size2(typ)
    }
    pub fn sizeof(&self, typ: &Type) -> u32 {
        self.backend.sizeof(typ)
    }

    pub fn int_ptr(&self, signed: bool) -> IRSize {
        self.backend.int_ptr(signed)
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
