use crate::parser::lifetimes::Lifetimes;
use std::collections::HashMap;
use syn::{parse, spanned::Spanned, Type};

pub type DataTypes = HashMap<String, Type>;

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

    // helper function for adding a new data type to a data descriptions struct
    fn add(&mut self, key: String, data_type: Type) -> Result<(), parse::Error> {
        // retrieve any lifetimes used in this data-type
        let lifetimes = Lifetimes::from_type(&data_type)?;

        // add the data to the collection
        self.data_types.insert(key.clone(), data_type);

        // if any new lifetimes were used in the type definition, we add those as well
        if !lifetimes.is_empty() {
            self.all_lifetimes.extend(&lifetimes);
            self.lifetimes.insert(key, lifetimes);
        }
        Ok(())
    }

    // helper function for collecting data types and adding them to a data descriptions struct
    pub fn collect(&mut self, key: String, data_type: Option<Type>) -> Result<(), parse::Error> {
        // check to see if there was every a previous data-type associated with this transition
        let prev = self.data_types.get(&key);

        // if there was a previous data definition for this key, may sure it is consistent
        if let Some(prev) = prev {
            if let Some(ref data_type) = data_type {
                if prev != &data_type.clone() {
                    return Err(parse::Error::new(
                        data_type.span(),
                        "This event's type does not match its previous definition.",
                    ));
                }
            } else {
                return Err(parse::Error::new(
                    data_type.span(),
                    "This event's type does not match its previous definition.",
                ));
            }
        }

        if let Some(data_type) = data_type {
            self.add(key, data_type)?;
        }
        Ok(())
    }
}
