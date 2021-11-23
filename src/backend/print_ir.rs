use super::ir::*;
use std::fmt;
use std::fmt::Display;

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
        match self {
            Imm(size, reg, value) => write!(f, "\t%{}=loadi {} #{}", reg, size, value),
            Ret(size, reg) => write!(f, "\tret {} %{}", size, reg),
        }
    }
}

impl Display for IRSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "i32")
    }
}
