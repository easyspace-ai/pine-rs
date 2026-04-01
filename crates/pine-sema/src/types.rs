//! Type system for Pine Script semantic analysis

use indexmap::IndexMap;
use std::fmt;

/// A field definition in a user-defined type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDef {
    /// Field name
    pub name: String,
    /// Field type
    pub ty: PineType,
    /// Default value expression (as string for now)
    pub default: Option<String>,
}

/// A method definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodDef {
    /// Method name
    pub name: String,
    /// Parameter types
    pub params: Vec<PineType>,
    /// Return type
    pub ret_ty: PineType,
}

/// A user-defined type definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDef {
    /// Type name
    pub name: String,
    /// Fields defined in this type
    pub fields: IndexMap<String, FieldDef>,
    /// Methods defined for this type
    pub methods: IndexMap<String, MethodDef>,
}

impl TypeDef {
    /// Create a new type definition
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: IndexMap::new(),
            methods: IndexMap::new(),
        }
    }

    /// Add a field to this type
    pub fn add_field(&mut self, name: impl Into<String>, ty: PineType, default: Option<String>) {
        let name_str = name.into();
        self.fields.insert(
            name_str.clone(),
            FieldDef {
                name: name_str,
                ty,
                default,
            },
        );
    }

    /// Get a field by name
    pub fn get_field(&self, name: &str) -> Option<&FieldDef> {
        self.fields.get(name)
    }

    /// Add a method to this type
    pub fn add_method(&mut self, name: impl Into<String>, params: Vec<PineType>, ret_ty: PineType) {
        let name_str = name.into();
        self.methods.insert(
            name_str.clone(),
            MethodDef {
                name: name_str,
                params,
                ret_ty,
            },
        );
    }

    /// Get a method by name
    pub fn get_method(&self, name: &str) -> Option<&MethodDef> {
        self.methods.get(name)
    }
}

/// Pine Script types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PineType {
    /// Integer type
    Int,
    /// Float type
    Float,
    /// Boolean type
    Bool,
    /// String type
    String,
    /// Color type
    Color,
    /// Series of some type
    Series(Box<PineType>),
    /// Array of some type
    Array(Box<PineType>),
    /// Matrix of some type
    Matrix(Box<PineType>),
    /// Map with key and value types
    Map(Box<PineType>, Box<PineType>),
    /// User-defined type
    Udt(String),
    /// Function type (params, return type)
    Function(Vec<PineType>, Box<PineType>),
    /// Unknown type (for inference)
    Unknown,
    /// Error type
    Error,
}

impl fmt::Display for PineType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PineType::Int => write!(f, "int"),
            PineType::Float => write!(f, "float"),
            PineType::Bool => write!(f, "bool"),
            PineType::String => write!(f, "string"),
            PineType::Color => write!(f, "color"),
            PineType::Series(inner) => write!(f, "series {}", inner),
            PineType::Array(inner) => write!(f, "array<{}>", inner),
            PineType::Matrix(inner) => write!(f, "matrix<{}>", inner),
            PineType::Map(key, value) => write!(f, "map<{}, {}>", key, value),
            PineType::Udt(name) => write!(f, "{}", name),
            PineType::Function(params, ret) => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            PineType::Unknown => write!(f, "unknown"),
            PineType::Error => write!(f, "error"),
        }
    }
}

impl PineType {
    /// Check if this type is a series type
    pub fn is_series(&self) -> bool {
        matches!(self, PineType::Series(_))
    }

    /// Get the inner type of a series
    pub fn series_inner(&self) -> Option<&PineType> {
        match self {
            PineType::Series(inner) => Some(inner),
            _ => None,
        }
    }

    /// Check if this is a numeric type (Int or Float)
    pub fn is_numeric(&self) -> bool {
        matches!(self, PineType::Int | PineType::Float)
    }
}
