use super::ast::*;
use std::fs::File;
use std::io::Write;

// This function prints the entire ast into graphviz format
pub fn print_graph(file: &str, ast: &dyn Graph) -> std::io::Result<()> {
    let mut buffer = File::create(file)?;
    let mut node_number = 0;
    writeln!(&mut buffer, "graph graphname {{")?;
    ast.graph(&mut buffer, &mut node_number, 0)?;
    writeln!(&mut buffer, "}}")?;
    Ok(())
}

// The graph trait is used to print the AST
// It is implemented for all AST types
// The node number is modified to give unique numbers to each printed node
// The parent is used to allow the children to add a connection
pub trait Graph {
    fn graph(
        &self,
        buffer: &mut dyn std::io::Write,
        node_number: &mut u32,
        parent: u32,
    ) -> std::io::Result<()>;
}

impl Graph for TranslationUnit {
    fn graph(
        &self,
        buffer: &mut dyn std::io::Write,
        node_number: &mut u32,
        _parent: u32,
    ) -> std::io::Result<()> {
        for declaration in &self.global_declarations {
            declaration.graph(buffer, node_number, 0)?;
        }
        Ok(())
    }
}

impl Graph for ExternalDeclaration {
    fn graph(
        &self,
        buffer: &mut dyn std::io::Write,
        node_number: &mut u32,
        _parent: u32,
    ) -> std::io::Result<()> {
        *node_number += 1;
        let number = *node_number;
        writeln!(buffer, "n{} [label=\"function {}\"]", number, self.name)?;
        if let Some(ref statements) = self.function_body {
            for statement in statements {
                statement.graph(buffer, node_number, number)?;
            }
        }
        Ok(())
    }
}

impl Graph for Statement {
    fn graph(
        &self,
        buffer: &mut dyn std::io::Write,
        node_number: &mut u32,
        parent: u32,
    ) -> std::io::Result<()> {
        *node_number += 1;
        let number = *node_number;
        use Statement::*;
        match self {
            Break { .. } => {
                writeln!(buffer, "n{} [label=\"break\"]", number)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
            }
            Continue { .. } => {
                writeln!(buffer, "n{} [label=\"continue\"]", number)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
            }

            Compound {
                span: _,
                statements,
            } => {
                writeln!(buffer, "n{} [label=\"<compound-statement>\"]", number)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
                for stmt in statements {
                    stmt.graph(buffer, node_number, number)?;
                }
            }

            Declaration {
                span: _,
                ident,
                decl_type: _,
                init,
            } => {
                writeln!(buffer, "n{} [label=\"declaration {}\"]", number, ident)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
                if let Some(init) = init {
                    init.graph(buffer, node_number, number)?;
                }
            }

            Empty(_) => {
                writeln!(buffer, "n{} [label=\"<empty>\"]", number)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
            }

            Expression {
                span: _,
                expression,
            } => {
                expression.graph(buffer, node_number, parent)?;
            }

            If {
                span: _,
                expression,
                statement,
                else_statement,
            } => {
                if let Some(else_state) = else_statement {
                    writeln!(buffer, "n{} [label=\"if-else\"]", number)?;
                    writeln!(buffer, "n{} -- n{}", parent, number)?;
                    expression.graph(buffer, node_number, number)?;
                    statement.graph(buffer, node_number, number)?;
                    else_state.graph(buffer, node_number, number)?;
                } else {
                    writeln!(buffer, "n{} [label=\"if\"]", number)?;
                    writeln!(buffer, "n{} -- n{}", parent, number)?;
                    expression.graph(buffer, node_number, number)?;
                    statement.graph(buffer, node_number, number)?;
                }
            }

            For {
                span: _,
                init,
                condition,
                expression,
                statement,
            } => {
                writeln!(buffer, "n{} [label=\"for\"]", number)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
                init.as_ref()
                    .map(|init| init.graph(buffer, node_number, number).unwrap());
                condition
                    .as_ref()
                    .map(|condition| condition.graph(buffer, node_number, number).unwrap());
                expression
                    .as_ref()
                    .map(|expression| expression.graph(buffer, node_number, number).unwrap());
                statement.graph(buffer, node_number, number)?;
            }

            Return {
                span: _,
                expression,
            } => {
                writeln!(buffer, "n{} [label=\"return\"]", number)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
                expression.graph(buffer, node_number, number)?;
            }

            While {
                span: _,
                expression,
                statement,
                do_while,
            } => {
                match do_while {
                    true => writeln!(buffer, "n{} [label=\"do-while\"]", number)?,
                    false => writeln!(buffer, "n{} [label=\"while\"]", number)?,
                }
                expression.graph(buffer, node_number, number)?;
                statement.graph(buffer, node_number, number)?;
            }
        }
        Ok(())
    }
}

impl Graph for Expression {
    fn graph(
        &self,
        buffer: &mut dyn std::io::Write,
        node_number: &mut u32,
        parent: u32,
    ) -> std::io::Result<()> {
        *node_number += 1;
        let number = *node_number;
        use ExpressionVariant::*;
        match &self.variant {
            ConstI(value) => {
                writeln!(buffer, "n{} [label=\"int {}\"]", number, value)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
            }

            CString(value) => {
                writeln!(buffer, "n{} [label=\"string \\\"{}\\\"\"]", number, value)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
            }

            Ident(name, ..) => {
                writeln!(buffer, "n{} [label=\"identifier {}\"]", number, name)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
            }

            Function(func, arguments) => {
                writeln!(buffer, "n{} [label=\"<function-call>\"]", number)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
                func.graph(buffer, node_number, number)?;
                for arg in arguments {
                    arg.graph(buffer, node_number, number)?;
                }
            }

            Unary(op, exp) => {
                writeln!(buffer, "n{} [label=\"{}\"]", number, op)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
                exp.graph(buffer, node_number, number)?;
            }

            Binary(op, left, right) => {
                writeln!(buffer, "n{} [label=\"{}\"]", number, op)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
                left.graph(buffer, node_number, number)?;
                right.graph(buffer, node_number, number)?;
            }

            Ternary(cond, left, right) => {
                writeln!(buffer, "n{} [label=\"?:\"]", number)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
                cond.graph(buffer, node_number, number)?;
                left.graph(buffer, node_number, number)?;
                right.graph(buffer, node_number, number)?;
            }

            Assign(left, right) => {
                writeln!(buffer, "n{} [label=\"=\"]", number)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
                left.graph(buffer, node_number, number)?;
                right.graph(buffer, node_number, number)?;
            }
        }
        Ok(())
    }
}
