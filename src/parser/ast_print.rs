use super::ast::*;
use super::r#type::Type;
use std::fmt;
use std::fmt::Display;

fn type2string(typ: &[Type]) -> String {
    let mut result = String::new();
    if typ.is_empty() {
        return result;
    }
    for i in (0..=(typ.len() - 1)).rev() {
        use Type::*;
        match &typ[i] {
            Int => result.push_str("int "),
            Name(name) => result.push_str(&name),
            Function(_) => {
                //Extend later when functions are fully implemented
                result.push_str(&format!("{}()", type2string(&typ[0..i])));
                break;
            }
        };
    }
    result
}

impl Display for TranslationUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for declaration in &self.global_declarations {
            writeln!(f, "{}", declaration)?;
        }
        Ok(())
    }
}

impl Display for ExternalDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", type2string(&self.ast_type))?;
        match &self.function_body {
            None => writeln!(f, ";")?,
            Some(body) => {
                writeln!(f, "{{")?;
                for statement in body {
                    write!(f, "{}", statement)?;
                }
                writeln!(f, "}}")?;
            }
        }
        Ok(())
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Statement::*;
        match self {
            Return {
                span: _,
                expression,
            } => writeln!(f, "return {};", expression)?,
        }
        Ok(())
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ExpressionVariant::*;
        match self.variant {
            ConstI(value) => write!(f, "{}", value)?,
        }
        Ok(())
    }
}
