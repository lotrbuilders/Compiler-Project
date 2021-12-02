use super::ir::*;
use std::fmt;
use std::fmt::Display;

// This prints the IR in an LLVM like format using the Display trait

impl Display for IRFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "define {} @{}() {{", self.return_size, self.name)?;
        for instruction in &self.instructions {
            writeln!(f, "{}", instruction)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl Display for IRInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use IRInstruction::*;
        let ins = self.to_type();
        match self {
            Imm(size, reg, value) => write!(f, "\t%{} = {} {} #{}", reg, ins, size, value),
            Add(size, result, left, right)
            | Sub(size, result, left, right)
            | Mul(size, result, left, right)
            | Div(size, result, left, right)
            | Xor(size, result, left, right)
            | Eq(size, result, left, right) => {
                write!(f, "\t%{} = {} {} %{}, %{}", result, ins, size, left, right)
            }

            Ret(size, reg) => write!(f, "\t{} {} %{}", ins, size, reg),
        }
    }
}

impl Display for IRType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use IRType::*;
        match self {
            Imm => write!(f, "loadi"),
            Add => write!(f, "add"),
            Sub => write!(f, "sub"),
            Mul => write!(f, "mul"),
            Div => write!(f, "div"),
            Xor => write!(f, "xor"),
            Eq => write!(f, "eq"),
            Ret => write!(f, "ret"),
        }
    }
}

impl Display for IRSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "i32")
    }
}
