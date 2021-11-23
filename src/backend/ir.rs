pub struct IRFunction {
    pub name: String,
    pub return_size: IRSize,
    pub instructions: Vec<IRInstruction>,
}

pub enum IRInstruction {
    Imm(IRSize, IRReg, i128),
    Ret(IRSize, IRReg),
}

type IRReg = u32;

pub enum IRSize {
    I32,
}
