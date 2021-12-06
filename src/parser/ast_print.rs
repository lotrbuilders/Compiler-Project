use super::ast::*;
use super::r#type::type2string;
use std::fmt;
use std::fmt::Display;

// This module implements the Display trait for the AST
// The print-out should be valid c code to allow for relexing and reparsing

// Allows the conversion of a type into a String representing the type as used in C

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
            Declaration {
                span: _,
                ident: _,
                decl_type,
                init,
            } => {
                write!(f, "{}", type2string(decl_type))?;
                if let Some(exp) = init {
                    writeln!(f, " = {};", exp)?;
                } else {
                    writeln!(f, ";")?;
                }
            }
            Expression {
                span: _,
                expression,
            } => writeln!(f, "{}", expression)?,
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
        match &self.variant {
            ConstI(value) => write!(f, "{}", value)?,
            Ident(name, _) => write!(f, "{}", name)?,

            Identity(exp) | Negate(exp) | BinNot(exp) | LogNot(exp) => {
                write!(
                    f,
                    "({} {})",
                    match &self.variant {
                        Identity(_) => '+',
                        Negate(_) => '-',
                        BinNot(_) => '~',
                        LogNot(_) => '!',
                        _ => unreachable!(),
                    },
                    exp
                )?;
            }

            Assign(left, right)
            | Add(left, right)
            | Subtract(left, right)
            | Multiply(left, right)
            | Divide(left, right) => {
                write!(
                    f,
                    "({} {} {})",
                    left,
                    match self.variant {
                        Assign(..) => '=',
                        Add(..) => '+',
                        Subtract(..) => '-',
                        Multiply(..) => '*',
                        Divide(..) => '/',
                        _ => unreachable!(),
                    },
                    right
                )?;
            }
        }
        Ok(())
    }
}
