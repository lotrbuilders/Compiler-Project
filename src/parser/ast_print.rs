use crate::table::StructTable;

use super::ast::*;
use super::r#type::StructType;
use std::fmt;
use std::fmt::Display;

// This module implements the Display trait for the AST
// The print-out should be valid c code to allow for relexing and reparsing

// Allows the conversion of a type into a String representing the type as used in C
pub trait ASTDisplay {
    fn fmt(&self, f: &mut fmt::Formatter, table: &StructTable) -> fmt::Result;
    fn fmt_braced(&self, f: &mut fmt::Formatter, table: &StructTable) -> fmt::Result {
        write!(f, "(")?;
        self.fmt(f, table)?;
        write!(f, ")")?;
        Ok(())
    }
    fn fmt_square(&self, f: &mut fmt::Formatter, table: &StructTable) -> fmt::Result {
        write!(f, "[")?;
        self.fmt(f, table)?;
        write!(f, "]")?;
        Ok(())
    }
}

pub struct PrintAst<'a, T>
where
    T: ASTDisplay,
{
    item: &'a T,
    table: &'a StructTable,
}

impl<'a, T> PrintAst<'a, T>
where
    T: ASTDisplay,
{
    pub fn new(item: &'a T, table: &'a StructTable) -> PrintAst<'a, T> {
        PrintAst { item, table }
    }
}

impl<'a, T> Display for PrintAst<'a, T>
where
    T: ASTDisplay,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let PrintAst { item, table } = *self;
        item.fmt(f, table)
    }
}

impl<'a> ASTDisplay for TranslationUnit {
    fn fmt(&self, f: &mut fmt::Formatter, table: &StructTable) -> fmt::Result {
        write!(f, "{}", table)?;
        for declaration in &self.global_declarations {
            declaration.fmt(f, table)?;
        }
        Ok(())
    }
}

impl<'a> ASTDisplay for ExternalDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter, table: &StructTable) -> fmt::Result {
        use super::TypeNode;
        let mut typ = self.ast_type.clone();
        typ.nodes.insert(0, TypeNode::Name(self.name.clone()));
        write!(f, "{}", typ)?;
        match &self.function_body {
            None => writeln!(f, ";")?,
            Some(body) => {
                writeln!(f, "{{")?;
                for statement in body {
                    statement.fmt(f, table)?;
                }
                writeln!(f, "}}")?;
            }
        }
        Ok(())
    }
}

impl<'a> ASTDisplay for Statement {
    fn fmt(&self, f: &mut fmt::Formatter, table: &StructTable) -> fmt::Result {
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
                    stmt.fmt(f, table)?;
                }
                writeln!(f, "}}")?;
            }

            Declaration {
                span: _,
                ident,
                decl_type,
                init,
            } => {
                use super::TypeNode;
                let mut typ = decl_type.clone();
                typ.nodes.insert(0, TypeNode::Name(ident.clone()));
                write!(f, "{}", typ)?;
                if let Some(exp) = init {
                    write!(f, " = ")?;
                    exp.fmt(f, table)?;
                }
                writeln!(f, ";")?;
            }

            Empty(_) => write!(f, ";")?,

            Expression {
                span: _,
                expression,
            } => {
                expression.fmt_braced(f, table)?;
                writeln!(f, ";")?;
            }

            If {
                span: _,
                expression,
                statement,
                else_statement,
            } => {
                write!(f, "if ",)?;
                expression.fmt_braced(f, table)?;
                statement.fmt(f, table)?;
                if let Some(statement) = else_statement {
                    writeln!(f, "else \n")?;
                    statement.fmt(f, table)?;
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
                    Some(init) => init.fmt(f, table)?,
                    None => write!(f, ";")?,
                }

                if let Some(cond) = condition {
                    cond.fmt(f, table)?;
                }
                write!(f, ";")?;

                match expression {
                    Some(init) => init.fmt(f, table)?,
                    None => (),
                }
                writeln!(f, ")")?;
                statement.fmt(f, table)?;
            }

            Return {
                span: _,
                ast_type: _,
                expression,
            } => {
                write!(f, "return ",)?;
                expression.fmt(f, table)?;
                writeln!(f, ";")?;
            }

            While {
                span: _,
                expression,
                statement,
                do_while: false,
            } => {
                writeln!(f, "while ")?;
                expression.fmt_braced(f, table)?;
                statement.fmt(f, table)?;
            }

            While {
                span: _,
                expression,
                statement,
                do_while: true,
            } => {
                write!(f, "do\n",)?;
                statement.fmt(f, table)?;
                write!(f, "while ")?;
                expression.fmt_braced(f, table)?;
                writeln!(f, ";")?;
            }
        }
        Ok(())
    }
}

impl<'a> ASTDisplay for Expression {
    fn fmt(&self, f: &mut fmt::Formatter, table: &StructTable) -> fmt::Result {
        use ExpressionVariant::*;
        match &self.variant {
            ConstI(value) => write!(f, "{} ", value)?,
            CString(value) => write!(f, "{:?} ", value)?,
            Ident(name, ..) => write!(f, "{} ", name)?,
            Sizeof(typ) => {
                write!(f, "sizeof ",)?;
                typ.fmt(f, table)?;
            }

            Function(func, arguments) => {
                write!(f, "(")?;
                func.fmt(f, table)?;
                write!(f, "(")?;
                if let Some(arg) = arguments.get(0) {
                    arg.fmt(f, table)?;
                }
                for arg in arguments.iter().skip(1) {
                    write!(f, ",")?;
                    arg.fmt(f, table)?;
                }
                write!(f, "))")?;
            }

            Member(exp, id, indirect) => {
                exp.fmt_braced(f, table)?;
                write!(f, "{}{}", if *indirect { "->" } else { "." }, id)?;
            }

            Unary(UnaryExpressionType::Cast, exp) => {
                write!(f, "({})", self.ast_type)?;
                exp.fmt_braced(f, table)?;
            }

            Unary(op, exp) => {
                write!(f, "{} ", op)?;
                exp.fmt_braced(f, table)?;
            }

            Binary(BinaryExpressionType::Index, left, right) => {
                left.fmt_braced(f, table)?;
                right.fmt_square(f, table)?;
            }

            Binary(op, left, right) => {
                left.fmt_braced(f, table)?;
                write!(f, " {} ", op)?;
                right.fmt_braced(f, table)?;
            }

            Ternary(cond, left, right) => {
                cond.fmt_braced(f, table)?;
                write!(f, "?")?;
                left.fmt_braced(f, table)?;
                write!(f, ":")?;
                right.fmt_braced(f, table)?;
            }

            Assign(left, right) => {
                left.fmt_braced(f, table)?;
                write!(f, " = ")?;
                right.fmt_braced(f, table)?;
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
                Cast => "cast",
            }
        )
    }
}

impl ASTDisplay for SizeofType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, table: &StructTable) -> fmt::Result {
        match self {
            SizeofType::Type(typ) => typ.fmt_braced(f, table),
            SizeofType::Expression(exp) => exp.fmt_braced(f, table),
        }
    }
}

impl ASTDisplay for StructType {
    fn fmt(&self, f: &mut fmt::Formatter, table: &StructTable) -> fmt::Result {
        use super::TypeNode;
        if self.members.is_some() {
            writeln!(f, "\n{{")?;
            for (name, typ) in self.members.as_ref().unwrap() {
                let typ = typ.clone();
                let name = vec![TypeNode::Name(name.clone())].into();
                let typ = typ.append(&name);
                writeln!(f, "{};\n", PrintAst::new(&typ, &table))?;
            }
            writeln!(f, "}}")?;
        }

        Ok(())
    }
}
