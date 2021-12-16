use super::ir::*;
use std::fmt::Display;
use std::fmt::{self};

// This prints the IR in an LLVM like format using the Display trait

fn fmt_argument(arguments: &IRArguments, f: &mut fmt::Formatter) -> fmt::Result {
    let mut iter = arguments.sizes.iter().zip(arguments.arguments.iter());
    let mut stack_arg = 0;
    if let Some((size, vreg)) = iter.next() {
        if let Some(vreg) = vreg {
            write!(f, "{} %{}", size, vreg)?;
        } else {
            write!(f, "{} ${}", size, stack_arg)?;
            stack_arg += 1;
        }
    }
    for (size, vreg) in iter {
        if let Some(vreg) = vreg {
            write!(f, ", {} %{}", size, vreg)?;
        } else {
            write!(f, "{} ${}", size, stack_arg)?;
            stack_arg += 1;
        }
    }
    Ok(())
}

impl Display for IRFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "define {} @{}(", self.return_size, self.name)?;
        fmt_argument(&self.arguments, f)?;
        writeln!(f, ") [")?;
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

impl Display for IRGlobal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.function {
            writeln!(f, "declaration {} @{}()", self.size, self.name)?;
        } else if let Some(value) = self.value {
            writeln!(f, "@{} = {} {}", self.name, self.size, value)?;
        } else {
            writeln!(f, "declaration {} @{}", self.size, self.name)?;
        }
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
            AddrG(size, reg, name) => write!(f, "\t%{} = {} {} @{}", reg, ins, size, name),
            Arg(size, reg, Some(index)) => write!(f, "\t%{} {} %{} for {}", ins, size, reg, index),
            Arg(size, reg, None) => write!(f, "\t%{} {} %{}", ins, size, reg),

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
            Call(size, result, name, arguments) => {
                write!(f, "\t%{} = {} call @{}({})", result, size, name, arguments)
            }
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
            AddrG => write!(f, "addrg"),
            Arg => write!(f, "arg"),
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

impl Display for IRArguments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.sizes.iter().zip(self.arguments.iter());
        if let Some((size, vreg)) = iter.next() {
            if let Some(vreg) = vreg {
                write!(f, "{} %{}", size, vreg)?;
            }
        }
        for (size, vreg) in iter {
            if let Some(vreg) = vreg {
                write!(f, ", {} %{}", size, vreg)?;
            }
        }
        Ok(())
    }
}
