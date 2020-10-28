// use std::collections::HashMap;

pub type Identifier = String;
pub type Index = usize;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    ArrayVal(Vec<Value>),
    I32Val(i32),
    F64Val(f64),
    // MapVal(HashMap<Identifier, Value>),
    NilVal,
    StringVal(String),
    UndefinedVal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Location {
    VariableLoc(Identifier),
    IndexLoc(Box<Location>, Index), // Arrays
    // KeyLoc(Identifier, Identifier), // Maps
}

// #[derive(Clone, Debug, PartialEq)]
// pub enum SliceLocation {
//     SimpleLoc(Location),
//     SliceLoc(Identifier, Index, Index),
// }

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    #[allow(dead_code)]
    NoOp,
    // Changes the display of a named value.
    Display(Location, DisplayType),
    Set(Location, Value),
    // Forget(Location),
    Push(Location, Value),
    Pop(Location),
    // Remove(Identifier, Index),
    // Copy(SliceLocation, Location),
    // Move(SliceLocation, Location),
    // Swap(Location, Location),
    // Tag(Location, Identifier),
    // Untag(Location, Identifier),
    // PushTag(Location, Identifier),
    // PopTag(Location),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DisplayType {
    Default,
    Tree,
}

impl Value {
    pub fn display_string(&self) -> String {
        match self {
            Value::ArrayVal(vec) => format!("[{}]", vec.iter().map(|v| v.display_string()).collect::<Vec<String>>().join(", ")),
            Value::I32Val(n) => n.to_string(),
            Value::F64Val(x) => x.to_string(),
            // Value::MapVal(m) => panic!("map val to_string {:?}", m),
            Value::NilVal => "nil".to_string(),
            Value::StringVal(s) => format!("{:?}", s),
            Value::UndefinedVal => "<undefined>".to_string(),
        }
    }

    pub fn is_simple(&self) -> bool {
        match self {
            Value::I32Val(_)
            | Value::F64Val(_)
            | Value::NilVal
            | Value::StringVal(_)
            | Value::UndefinedVal => true,

            Value::ArrayVal(_) => false,
        }
    }
}
