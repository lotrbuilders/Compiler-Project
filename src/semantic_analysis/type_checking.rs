use super::SemanticAnalyzer;
use crate::error;
use crate::parser::ast::Expression;
use crate::parser::r#type::Type;
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
