use super::ast::*;
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
        write!(f, "{}", self.ast_type)?;
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
            Break { .. } => writeln!(f, "break;")?,

            Continue { .. } => writeln!(f, "continue;")?,

            Compound {
                span: _,
                statements,
            } => {
                writeln!(f, "{{")?;
                for stmt in statements {
                    write!(f, "{}", stmt)?;
                }
                writeln!(f, "}}")?;
            }

            Declaration {
                span: _,
                ident: _,
                decl_type,
                init,
            } => {
                write!(f, "{}", decl_type)?;
                if let Some(exp) = init {
                    writeln!(f, " = {};", exp)?;
                } else {
                    writeln!(f, ";")?;
                }
            }

            Empty(_) => write!(f, ";")?,

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

            For {
                span: _,
                init,
                condition,
                expression,
                statement,
            } => {
                write!(f, "for (")?;
                match init {
                    Some(init) => write!(f, "{}", init)?,
                    None => write!(f, ";")?,
                }

                if let Some(init) = condition {
                    write!(f, "{}", init)?;
                }
                write!(f, ";")?;

                match expression {
                    Some(init) => writeln!(f, "{})", init)?,
                    None => writeln!(f, ")")?,
                }

                writeln!(f, "{}", statement)?;
            }

            Return {
                span: _,
                expression,
            } => writeln!(f, "return {};", expression)?,

            While {
                span: _,
                expression,
                statement,
                do_while: false,
            } => {
                writeln!(f, "while ({})", expression)?;
                writeln!(f, "{}", statement)?;
            }

            While {
                span: _,
                expression,
                statement,
                do_while: true,
            } => {
                write!(f, "do\n{}", statement)?;
                writeln!(f, "while ({});", expression)?;
            }
        }
        Ok(())
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ExpressionVariant::*;
        match &self.variant {
            ConstI(value) => write!(f, "{}", value)?,
            Ident(name, ..) => write!(f, "{}", name)?,

            Function(func, arguments) => {
                write!(f, "({}(", func)?;
                if let Some(arg) = arguments.get(0) {
                    write!(f, "{}", arg)?;
                }
                for arg in arguments.iter().skip(1) {
                    write!(f, ",{}", arg)?;
                }
                write!(f, "))")?;
            }

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
                BinOr => "|",
                BinAnd => "&",
                LogOr => "||",
                LogAnd => "&&",
                Comma => ",",
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
                Deref => '*',
                Address => '&',
            }
        )
    }
}
