use super::SemanticAnalyzer;
use crate::error;
use crate::parser::ast::Expression;
use crate::parser::r#type::Type;
use crate::semantic_analysis::type_class::TypeClass;
use crate::span::Span;

pub fn check_arguments_function(
    analyzer: &mut SemanticAnalyzer,
    span: &Span,
    function_type: &Type,
    arguments: &Vec<Expression>,
) {
    if !Type::is_function(function_type) {
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

    if argument_type.len() != arguments.len() {
        analyzer.errors.push(error!(
            span,
            "The amount of arguments in the function({}) does not match the amount supplied({})",
            argument_type.len(),
            arguments.len()
        ))
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
) -> Type {
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
        return Type::error();
    }

    // Get the definition of the type and check if it is qualified(fully defined)
    let index = struct_type.get_struct_index();
    let struct_def = &(&analyzer.struct_table)[index];
    if !struct_def.is_qualified() {
        analyzer.errors.push(error!(
            span,
            "{} is unqualifed and cannot be accesed by {}", struct_type, id
        ));
        return Type::error();
    }

    // Check if the id matches any known members
    // (Iteration is probably faster then hashing here as n is generally small)
    for (member, typ) in struct_def.members.as_ref().unwrap() {
        if member == id {
            return typ.clone();
        }
    }

    // If we do not match any members then we have reached an error
    analyzer.errors.push(error!(
        span,
        "{} does not have a member named {}", struct_type, id
    ));
    Type::error()
}
