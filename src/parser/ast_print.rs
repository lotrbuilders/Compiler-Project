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
            } => writeln!(f, "{};", expression)?,
            If {
                span: _,
                expression,
                statement,
                else_statement,
            } => {
                writeln!(f, "if ({})", expression)?;
                writeln!(f, "{}", statement)?;
                if let Some(statement) = else_statement {
                    writeln!(f, "else \n{}", statement)?;
                }
            }
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

            Unary(op, exp) => {
                write!(f, "({} {})", op, exp)?;
            }

            Binary(op, left, right) => {
                write!(f, "({} {} {})", left, op, right)?;
            }

            Ternary(cond, left, right) => {
                write!(f, "({} ? {} : {})", cond, left, right)?;
            }

            Assign(left, right) => {
                write!(f, "({} = {})", left, right)?;
            }
        }
        Ok(())
    }
}

impl Display for BinaryExpressionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BinaryExpressionType::*;
        write!(
            f,
            "{}",
            match self {
                Add => "+",
                Subtract => "-",
                Multiply => "*",
                Divide => "/",
                Equal => "==",
                Inequal => "!=",
                Less => "<",
                LessEqual => "<=",
                Greater => ">",
                GreaterEqual => ">=",
            }
        )
    }
}

impl Display for UnaryExpressionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UnaryExpressionType::*;
        write!(
            f,
            "{}",
            match self {
                Identity => '+',
                Negate => '-',
                BinNot => '~',
                LogNot => '!',
            }
        )
    }
}
