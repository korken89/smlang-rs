use std::collections::HashMap;
use syn::{Lifetime, Type};

pub type DataTypes = HashMap<String, Type>;
pub type Lifetimes = Vec<Lifetime>;

#[derive(Debug)]
pub struct DataDefinitions {
    pub data_types: DataTypes,
    pub all_lifetimes: Lifetimes,
    pub lifetimes: HashMap<String, Lifetimes>,
}

impl DataDefinitions {
    pub fn new() -> Self {
        Self {
            data_types: DataTypes::new(),
            all_lifetimes: Lifetimes::new(),
            lifetimes: HashMap::new(),
        }
    }
}
