use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Bool,
    Float,
    String,
    // Null,
    Void,
    Unresolved,
    Error,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let type_name = match self {
            Type::Int => "int",
            Type::Bool => "bool",
            Type::Float => "float",
            Type::String => "string",
            // Type::Null => "null",
            Type::Unresolved => "unresolved",
            Type::Void => "void",
            Type::Error => "?",
        };

        write!(f, "{}", type_name)
    }
}

impl Type {
    pub fn is_assignable_to(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::Float, Type::Int) => true,
            (Type::Int, Type::Float) => true,
            (Type::String, Type::String) => true,
            (Type::String, Type::Int) => false,
            (Type::Bool, Type::Bool) => true,
            (Type::Error, _) => true,
            (_, Type::Error) => true,
            _ => false,
        }
    }

    pub fn from_str(s: &str) -> Option<Type> {
        match s {
            "int" => Some(Type::Int),
            "float" => Some(Type::Float),
            "string" => Some(Type::String),
            "bool" => Some(Type::Bool),
            "void" => Some(Type::Void),
            _ => None,
        }
    }
}
