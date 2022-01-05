use super::SemanticAnalyzer;
use crate::error;
use crate::parser::ast::{ASTStruct, ASTType, ASTTypeNode, Expression};
use crate::parser::r#type::{StructType, Type, TypeNode};
use crate::semantic_analysis::analysis::Analysis;
use crate::semantic_analysis::type_class::TypeClass;
use crate::span::Span;

pub fn check_arguments_function(
    analyzer: &mut SemanticAnalyzer,
    span: &Span,
    function_type: &Type,
    arguments: &Vec<Expression>,
) {
    if !Type::is_callable(function_type) {
        analyzer
            .errors
            .push(error!(span, "Cannot call '{}'", function_type));
        return;
    }
    let argument_type = Type::get_function_arguments(function_type).unwrap();

    // Functions that do  have unspecified arguments are per definition correctly called
    if argument_type.len() == 0 {
        return;
    }

    // Functions can have a special void argument, which means that function actually has no arguments
    if argument_type[0].is_void() {
        if arguments.len() != 0 {
            analyzer.errors.push(error!(
                span,
                "The amount of arguments in the function(0) does not match the amount supplied({})",
                arguments.len()
            ))
        }
        return;
    }

    if argument_type.len() != arguments.len() {
        analyzer.errors.push(error!(
            span,
            "The amount of arguments in the function({}) does not match the amount supplied({})",
            argument_type.len(),
            arguments.len()
        ))
    }
    for (argument, argument_type) in arguments.iter().zip(argument_type.iter()) {
        analyzer.assert_compatible(span, &argument.ast_type, argument_type)
    }
}

pub fn compare_arguments(
    analyzer: &mut SemanticAnalyzer,
    span: &Span,
    name: &String,
    lhs: &Type,
    rhs: &Type,
) {
    if lhs.get_function_arguments() != rhs.get_function_arguments() {
        analyzer.errors.push(error!(
            span,
            "Global {} previously defined with {} is redefined with {}",
            name,
            lhs.get_function_arguments()
                .map(|a| format!("function arguments '{:?}'", a))
                .unwrap_or("no function arguments".to_string()),
            rhs.get_function_arguments()
                .map(|a| format!("function arguments '{:?}'", a))
                .unwrap_or("no function arguments".to_string())
        ));
    }
}

pub fn compare_return_types(
    analyzer: &mut SemanticAnalyzer,
    span: &Span,
    name: &String,
    lhs: &Type,
    rhs: &Type,
) {
    if lhs.get_return_type() != rhs.get_return_type() {
        analyzer.errors.push(error!(
            span,
            "Global {} previously defined as '{}' is redefined as '{}'",
            name,
            lhs.get_return_type()
                .map(|t| t.into())
                .unwrap_or(Type::empty()),
            rhs.get_return_type()
                .map(|t| t.into())
                .unwrap_or(Type::empty())
        ));
    }
}

pub fn check_member_type(
    analyzer: &mut SemanticAnalyzer,
    span: &Span,
    ast_type: &Type,
    id: &String,
    indirect: bool,
) -> (Type, u16) {
    //Check and account for possible indirection
    let struct_type = if indirect {
        let ast_type = ast_type.array_promotion();
        analyzer.assert_in(span, &ast_type, TypeClass::Pointer);
        ast_type.deref()
    } else {
        ast_type.clone()
    };

    //Check if we are even processing a struct
    if !struct_type.is_struct() {
        analyzer
            .errors
            .push(error!(span, "Expected a struct, but found {}", ast_type));
        return (Type::error(), 0);
    }

    // Get the definition of the type and check if it is qualified(fully defined)
    let index = struct_type.get_struct_index();
    let struct_def = &(&analyzer.struct_table)[index];
    if !struct_def.is_qualified() {
        analyzer.errors.push(error!(
            span,
            "{} is unqualifed and cannot be accesed by {}", struct_type, id
        ));
        return (Type::error(), 0);
    }

    // Check if the id matches any known members
    // (Iteration is probably faster then hashing here as n is generally small)
    for ((member, typ), i) in struct_def.members.as_ref().unwrap().iter().zip(0..) {
        if member == id {
            return (typ.clone(), i);
        }
    }

    // If we do not match any members then we have reached an error
    analyzer.errors.push(error!(
        span,
        "{} does not have a member named {}", struct_type, id
    ));
    (Type::error(), 0)
}

impl ASTType {
    pub fn to_type(&mut self, analyzer: &mut SemanticAnalyzer) -> Type {
        use super::ASTTypeNode::*;
        use TypeNode::*;
        type AST = ASTTypeNode;
        let mut declarator = Vec::new();
        let mut type_specifiers = Vec::new();
        for entry in &mut self.list {
            match entry {
                Simple(t @ (Char | Int | Long | Short | Void)) => {
                    type_specifiers.push(t.clone());
                }
                Simple(Pointer) => declarator.push(Pointer),
                Simple(_) => unreachable!(),
                AST::Name(_) => (),
                AST::Struct(s) => {
                    type_specifiers.push(s.to_type(&self.span, analyzer));
                }
                AST::Function(arguments) => {
                    let (arguments, _) = ASTType::tranform_function_arguments(arguments, analyzer);
                    analyzer.assert_function_arguments(&self.span, &arguments);
                    declarator.push(TypeNode::Function(Box::new(arguments)));
                }
                AST::Array(exp) => {
                    exp.analyze(analyzer);
                    exp.force_const_eval(analyzer);
                    let value = exp.get_const_value();
                    if value.is_negative() {
                        analyzer
                            .errors
                            .push(error!(self.span, "Size of array must be positive"));
                    }
                    let value = std::cmp::max(value, 0);
                    declarator.push(TypeNode::Array(value as usize));
                }
            }
        }
        let base_type = analyzer.check_declaration_specifiers(&self.span, &type_specifiers);
        let declarator: Type = declarator.into();
        let typ = Type::combine(base_type, declarator);
        typ
    }
    pub fn get_function_arguments(
        &mut self,
        analyzer: &mut SemanticAnalyzer,
    ) -> Vec<(Type, Option<String>)> {
        type AST = ASTTypeNode;
        let mut arguments = None;
        log::debug!("get function arguments of {:?}", self);
        for entry in &mut self.list {
            match entry {
                AST::Name(_) => continue,
                AST::Simple(TypeNode::Pointer) => continue,
                AST::Function(args) => {
                    arguments = Some(args);
                    break;
                }
                t => {
                    log::error!("unexpected {:?}", t);
                    unreachable!()
                }
            }
        }
        let arguments = arguments.unwrap();
        let (types, names) = ASTType::tranform_function_arguments(arguments, analyzer);
        types.into_iter().zip(names.into_iter()).collect()
    }
    fn tranform_function_arguments(
        arguments: &mut Vec<ASTType>,
        analyzer: &mut SemanticAnalyzer,
    ) -> (Vec<Type>, Vec<Option<String>>) {
        arguments
            .iter_mut()
            .map(|ast| {
                let name = ast.get_name();
                let typ = ast.to_type(analyzer);
                (typ, name)
            })
            .unzip()
    }
}

impl ASTStruct {
    fn to_type(&mut self, span: &Span, analyzer: &mut SemanticAnalyzer) -> TypeNode {
        let name = self.name.as_ref();
        if self.members.is_none() {
            let name = name.unwrap();
            if analyzer.struct_table.contains(name) {
                let index = analyzer.struct_table.get_index(name).unwrap();
                return TypeNode::Struct(index);
            } else {
                let index = analyzer.struct_table.try_insert(Some(name)).unwrap();
                return TypeNode::Struct(index);
            }
        }

        // If a struct is already in the table and already qualified(definied) we raise an error
        if name.is_some()
            && analyzer.struct_table.contains(name.unwrap())
            && analyzer
                .struct_table
                .get(name.unwrap())
                .unwrap()
                .is_qualified()
        {
            analyzer
                .errors
                .push(error!(span, "Struct {} redefined", name.unwrap()));
            return TypeNode::error();
        }

        // If the struct is not yet defined, insert it
        // Otherwise we can get the index from the struct table
        let index = analyzer
            .struct_table
            .try_insert(name)
            .or_else(|_| -> Result<usize, ()> {
                Ok(analyzer.struct_table.get_index(name.unwrap()).unwrap())
            })
            .unwrap();

        let ast_members = self.members.as_mut().unwrap();
        let mut members = Vec::new();
        for member in ast_members {
            let name = member.get_name();
            if name.is_none() {
                analyzer
                    .errors
                    .push(error!(span, "Missing member name in struct definition"));
                continue;
            }

            let name = name.unwrap();
            let typ = member.to_type(analyzer);
            if !typ.is_qualified(&analyzer.struct_table) {
                analyzer.errors.push(error!(
                    span,
                    "Missing member {} is not qualified in struct definition", name
                ));
                continue;
            }
            members.push((name, typ))
        }

        let entry = StructType {
            name: self.name.clone(),
            members: Some(members),
        };
        analyzer
            .struct_table
            .qualify(&analyzer.type_info, index, entry);

        TypeNode::Struct(index)
    }
}

impl SemanticAnalyzer {
    fn check_declaration_specifiers(&mut self, span: &Span, typ: &[TypeNode]) -> Type {
        use TypeNode::*;
        let mut type_specifier = None;
        let mut int_seen = false;
        for node in typ {
            match node {
                TypeNode::Int => {
                    if int_seen {
                        self.invalid_type(span, &typ);
                    } else if let Some(TypeNode::Long | TypeNode::Short) = type_specifier {
                        int_seen = true;
                    } else if type_specifier.is_none() {
                        type_specifier = Some(Int);
                        int_seen = true;
                    } else {
                        self.invalid_type(span, typ);
                    }
                }
                t @ (TypeNode::Long | TypeNode::Short) => {
                    if let Some(TypeNode::Int) | None = type_specifier {
                        type_specifier = Some(t.clone());
                    } else {
                        self.invalid_type(span, &typ);
                    }
                }
                TypeNode::Struct(..) | TypeNode::Char | TypeNode::Void => {
                    if let Some(_) = type_specifier {
                        self.invalid_type(span, &typ);
                    }
                    type_specifier = Some(node.clone());
                }

                TypeNode::Function(..) | TypeNode::Array(..) | TypeNode::Pointer => unreachable!(),
            }
        }
        vec![type_specifier.expect("failure to check for type specifer")].into()
    }
    fn invalid_type(&mut self, span: &Span, typ: &[TypeNode]) {
        let typ: Type = typ.into();
        self.errors
            .push(error!(span, "Invalid type specifiers provided: {}", typ));
    }
}
