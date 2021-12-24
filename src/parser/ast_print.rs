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
                decl_type: _,
                ast_type,
                init,
            } => {
                write!(f, "{}", ast_type)?;
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
                ast_type: _,
                expression,
            } => match expression {
                Some(expression) => writeln!(f, "return {};", expression)?,
                None => writeln!(f, "return;")?,
            },

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
            CString(value) => write!(f, "\"{}\"", value)?,
            Ident(name, ..) => write!(f, "{}", name)?,
            Sizeof(typ) => write!(f, "sizeof {}", typ)?,

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

            Member(exp, id, indirect, _) => {
                write!(f, "({}){}{}", exp, if *indirect { "->" } else { "." }, id)?;
            }

            Cast(exp, typ) => {
                write!(f, "(({}){})", typ, exp)?;
            }

            Unary(op, exp) => {
                write!(f, "({} {})", op, exp)?;
            }

            Binary(BinaryExpressionType::Index, left, right) => {
                write!(f, "({}[{}])", left, right)?;
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
                Index => "[]",
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
                Identity => "+",
                Negate => "-",
                BinNot => "~",
                LogNot => "!",
                Deref => "*",
                Address => "&",
            }
        )
    }
}

impl Display for SizeofType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SizeofType::Type(typ, _) => write!(f, "({})", typ),
            SizeofType::Expression(exp) => write!(f, "{}", exp),
        }
    }
}

impl Display for ASTType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_prefix_type(&self.list, f)?;
        format_type(&self.list, f)
    }
}

fn format_prefix_type(typ: &[ASTTypeNode], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for i in 0..typ.len() {
        use super::TypeNode::*;
        use ASTTypeNode::*;
        type AST = ASTTypeNode;
        match &typ[i] {
            Simple(Char) => write!(f, "char ")?,
            Simple(Int) => write!(f, "int ")?,
            Simple(Long) => write!(f, "long ")?,
            Simple(Short) => write!(f, "short ")?,
            Simple(Void) => write!(f, "void ")?,
            AST::Struct(s) => {
                write!(f, "struct {} ", s.name.clone().unwrap_or_default())?;
                if let Some(members) = &s.members {
                    writeln!(f, "{{")?;
                    for member in members {
                        writeln!(f, "{};", member)?;
                    }
                    writeln!(f, "}}")?;
                }
            }
            _ => (),
        }
    }
    Ok(())
}

fn format_type(typ: &[ASTTypeNode], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if typ.is_empty() {
        return Ok(());
    }

    for i in (0..typ.len()).rev() {
        use super::TypeNode::*;
        use ASTTypeNode::*;
        type AST = ASTTypeNode;
        match &typ[i] {
            Simple(Char | Int | Long | Short | Void) => (),

            Simple(Pointer) => write!(f, "* ")?,
            Simple(t) => {
                log::error!("unexpected type node {:?}", t);
                unreachable!()
            }
            AST::Name(name) => write!(f, "{} ", name)?,
            AST::Function(arguments) => {
                //Extend later when functions are fully implemented
                format_type(&typ[0..i], f)?;
                write!(f, "(")?;
                if let Some(arg) = arguments.get(0) {
                    write!(f, "{}", arg)?;
                }
                for arg in arguments.iter().skip(1) {
                    write!(f, ", {}", arg)?;
                }
                write!(f, ")")?;
                break;
            }
            AST::Array(size) => {
                //Extend later when functions are fully implemented
                format_type(&typ[0..i], f)?;
                write!(f, "[{}]", size)?;
                break;
            }
            AST::Struct(..) => (),
        };
    }
    Ok(())
}
