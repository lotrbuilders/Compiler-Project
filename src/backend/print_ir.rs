use super::ir::*;
use std::fmt::Display;
use std::fmt::{self};

// This prints the IR in an LLVM like format using the Display trait

impl Display for IRFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "define {} @{}() [", self.return_size, self.name)?;
        for (local, i) in self.variables.iter().zip(0usize..) {
            writeln!(f, "\t${}: {}", i, local)?;
        }
        writeln!(f, "] {{")?;

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
            AddrL(size, reg, value) => write!(f, "\t%{} = {} {} ${}", reg, ins, size, value),

            Load(size, reg, addr) => write!(f, "\t%{} = {} {} [%{}]", reg, ins, size, addr),
            Store(size, reg, addr) => write!(f, "\t{} {} %{}, [%{}]\n", ins, size, reg, addr),

            Add(size, result, left, right)
            | Sub(size, result, left, right)
            | Mul(size, result, left, right)
            | Div(size, result, left, right)
            | Xor(size, result, left, right)
            | Or(size, result, left, right)
            | And(size, result, left, right)
            | Eq(size, result, left, right)
            | Ne(size, result, left, right)
            | Lt(size, result, left, right)
            | Le(size, result, left, right)
            | Gt(size, result, left, right)
            | Ge(size, result, left, right) => {
                write!(f, "\t%{} = {} {} %{}, %{}", result, ins, size, left, right)
            }

            Jcc(size, left, label) => write!(f, "\tjcc {} %{} L{}", size, left, label),
            Jnc(size, left, label) => write!(f, "\tjnc {} %{} L{}", size, left, label),
            Jmp(label) => write!(f, "\tjmp L{}", label),
            Label(Some(phi), label) => write!(f, "L{}:\t{}", label, phi),
            Label(None, label) => write!(f, "L{}:", label),
            PhiSrc(label) => write!(f, "\tphisrc L{}:", label),
            Phi(phi) => write!(f, "\t{}", phi),

            Ret(size, reg) => write!(f, "\t{} {} %{}", ins, size, reg),
        }
    }
}

impl Display for IRType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use IRType::*;
        match self {
            Imm => write!(f, "loadi"),
            AddrL => write!(f, "addrl"),
            Load => write!(f, "load"),
            Store => write!(f, "store"),
            Add => write!(f, "add"),
            Sub => write!(f, "sub"),
            Mul => write!(f, "mul"),
            Div => write!(f, "div"),
            Xor => write!(f, "xor"),
            Or => write!(f, "or"),
            And => write!(f, "and"),
            Eq => write!(f, "eq"),
            Ne => write!(f, "ne"),
            Lt => write!(f, "lt"),
            Le => write!(f, "le"),
            Gt => write!(f, "gt"),
            Ge => write!(f, "ge"),
            Ret => write!(f, "ret"),
            _ => unreachable!(),
        }
    }
}

impl Display for IRSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::S32 => write!(f, "s32"),
            Self::I32 => write!(f, "i32"),
            Self::P => write!(f, "p"),
            //_ => unreachable!(),
        }
    }
}

impl Display for IRPhi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} = phi  {:?} [ ", self.targets, self.size)?;
        for (label, registers) in self.locations.iter().zip(self.sources.iter()) {
            write!(f, "L{} {:?} ", label, registers)?;
        }
        write!(f, " ]")?;
        Ok(())
    }
}
