use crate::{
    backend::{ir::*, Backend},
    parser::{ast::BinaryExpressionType, Type},
};

pub struct EvaluationContext<'a> {
    pub vreg_counter: u32,
    pub label_counter: u32,
    pub variables: Vec<IRSize>,
    pub unfixed_continue: Vec<(usize, u32)>,
    pub unfixed_break: Vec<(usize, u32)>,
    pub loop_depth: u32,
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

impl<'a> EvaluationContext<'a> {
    pub fn get_size(&self, typ: &Type) -> IRSize {
        self.backend.get_size(&typ.nodes[0])
    }
    pub fn sizeof(&self, size: IRSize) -> u32 {
        match size {
            IRSize::S8 => 1,
            IRSize::S16 => 2,
            IRSize::S32 => 4,
            IRSize::S64 => 8,
            IRSize::P => self.backend.sizeof_pointer(),
        }
    }
    pub fn int_ptr(&self, signed: bool) -> IRSize {
        assert!(signed); //Unsigned integers are currently unsupported
        match self.backend.sizeof_pointer() {
            8 => IRSize::S16,
            4 => IRSize::S32,
            2 => IRSize::S64,
            _ => unreachable!(),
        }
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
