use super::ast::*;
use std::fs::File;
use std::io::Write;

pub fn print_graph(file: &str, ast: &dyn Graph) -> std::io::Result<()> {
    let mut buffer = File::create(file)?;
    let mut node_number = 0;
    writeln!(&mut buffer, "graph graphname {{")?;
    ast.graph(&mut buffer, &mut node_number, 0)?;
    writeln!(&mut buffer, "}}")?;
    Ok(())
}

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
            Return {
                span: _,
                expression,
            } => {
                writeln!(buffer, "n{} [label=\"return\"]", number)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
                expression.graph(buffer, node_number, number)?;
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
        match self.variant {
            ConstI(value) => {
                writeln!(buffer, "n{} [label=\"int {}\"]", number, value)?;
                writeln!(buffer, "n{} -- n{}", parent, number)?;
            }
        }
        Ok(())
    }
}
