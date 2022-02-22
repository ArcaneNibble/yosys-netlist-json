use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};

/// Legal values for the direction of a port on a module
#[derive(Copy, Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum PortDirection {
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "output")]
    Output,
    #[serde(rename = "inout")]
    InOut,
}

/// Special constant bit values
#[derive(Copy, Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum SpecialBit {
    /// Constant 0
    #[serde(rename = "0")]
    _0,
    /// Constant 1
    #[serde(rename = "1")]
    _1,
    /// Constant X (invalid)
    #[serde(rename = "x")]
    X,
    /// Constant Z (tri-state)
    #[serde(rename = "z")]
    Z,
}

#[cfg(feature = "slog")]
impl slog::Value for SpecialBit {
    fn serialize(
        &self,
        _record: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        match self {
            &SpecialBit::_0 => serializer.emit_str(key, "0"),
            &SpecialBit::_1 => serializer.emit_str(key, "1"),
            &SpecialBit::X => serializer.emit_str(key, "x"),
            &SpecialBit::Z => serializer.emit_str(key, "z"),
        }
    }
}

/// A number representing a single bit of a wire
#[derive(Copy, Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum BitVal {
    /// An actual signal number
    N(usize),
    /// A special constant value
    S(SpecialBit),
}

#[cfg(feature = "slog")]
impl slog::Value for BitVal {
    fn serialize(
        &self,
        record: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        match self {
            &BitVal::N(n) => serializer.emit_usize(key, n),
            &BitVal::S(s) => s.serialize(record, key, serializer),
        }
    }
}

/// The value of an attribute/parameter
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum AttributeVal {
    /// Numeric attribute value
    N(usize),
    /// String attribute value
    S(String),
}

impl AttributeVal {
    pub fn to_number(&self) -> Option<usize> {
        match self {
            &AttributeVal::N(n) => Some(n),
            &AttributeVal::S(ref s) => {
                // If it's an empty string, the value was zero
                if s.len() == 0 {
                    Some(0)
                } else {
                    usize::from_str_radix(s, 2).ok()
                }
            }
        }
    }

    pub fn to_string_if_string(&self) -> Option<&str> {
        match self {
            &AttributeVal::N(_) => None,
            &AttributeVal::S(ref s) => {
                if s.len() == 0 {
                    // If it's an empty string then it wasn't originally a string
                    None
                } else if s
                    .find(|c| !(c == '0' || c == '1' || c == 'x' || c == 'z'))
                    .is_none()
                {
                    // If it only contains 01xz, then it wasn't originally a string
                    None
                } else {
                    if *s.as_bytes().last().unwrap() == b' ' {
                        // If the last character is a space, drop it
                        Some(s.split_at(s.len() - 1).0)
                    } else {
                        Some(s)
                    }
                }
            }
        }
    }
}

#[cfg(feature = "slog")]
impl slog::Value for AttributeVal {
    fn serialize(
        &self,
        _record: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        match self {
            &AttributeVal::N(n) => serializer.emit_usize(key, n),
            &AttributeVal::S(ref s) => serializer.emit_str(key, s),
        }
    }
}

/// Represents an entire .json file used by Yosys
#[derive(Clone, Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
pub struct Netlist {
    /// The program that created this file.
    #[serde(default)]
    pub creator: String,
    /// A map from module names to module objects contained in this .json file
    #[serde(default)]
    pub modules: HashMap<String, Module>,
}

/// Represents one module in the Yosys hierarchy
#[derive(Clone, Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
pub struct Module {
    /// Module attributes (Verilog `(* attr *)`)
    #[serde(default)]
    pub attributes: HashMap<String, AttributeVal>,
    /// Module parameter (Verilog `parameter`) default values
    #[serde(default)]
    pub parameter_default_values: HashMap<String, AttributeVal>,
    /// Module ports (interfaces to other modules)
    #[serde(default)]
    pub ports: HashMap<String, Port>,
    /// Module cells (objects inside this module)
    #[serde(default)]
    pub cells: HashMap<String, Cell>,
    /// Module netnames (names of wires in this module)
    #[serde(default)]
    pub netnames: HashMap<String, Netname>,
}

/// Represents a port on a module
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Port {
    /// Port direction
    pub direction: PortDirection,
    /// Bit value(s) representing the wire(s) connected to this port
    pub bits: Vec<BitVal>,
}

/// Represents a cell in a module
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Cell {
    /// Indicates an internal/auto-generated name that starts with `$`
    #[serde(default)]
    pub hide_name: usize,
    /// Name of the type of this cell
    #[serde(rename = "type")]
    pub cell_type: String,
    /// Parameters specified on this cell
    #[serde(default)]
    pub parameters: HashMap<String, AttributeVal>,
    /// Attributes specified on this cell
    #[serde(default)]
    pub attributes: HashMap<String, AttributeVal>,
    /// The direction of the ports on this cell
    #[serde(default)]
    pub port_directions: HashMap<String, PortDirection>,
    /// Bit value(s) representing the wire(s) connected to the inputs/outputs of this cell
    pub connections: HashMap<String, Vec<BitVal>>,
}

/// Represents the name of a net in a module
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Netname {
    /// Indicates an internal/auto-generated name that starts with `$`
    #[serde(default)]
    pub hide_name: usize,
    /// Bit value(s) that should be given this name
    pub bits: Vec<BitVal>,
    /// Attributes for this netname
    #[serde(default)]
    pub attributes: HashMap<String, AttributeVal>,
}

impl Netlist {
    /// Read netlist data from a reader
    pub fn from_reader<R: Read>(reader: R) -> Result<Netlist, serde_json::Error> {
        serde_json::from_reader(reader)
    }

    /// Read netlist data from a slice containing the bytes from a Yosys .json file
    pub fn from_slice(input: &[u8]) -> Result<Netlist, serde_json::Error> {
        serde_json::from_slice(input)
    }

    /// Serialize to a String
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize to a writer
    pub fn to_writer<W: Write>(&self, writer: W) -> Result<(), serde_json::Error> {
        serde_json::to_writer(writer, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn super_empty_json() {
        let result = Netlist::from_slice(
            br#"
            {}"#,
        )
        .unwrap();
        assert_eq!(result.creator, "");
        assert_eq!(result.modules.len(), 0);
    }

    #[test]
    fn empty_json() {
        let result = Netlist::from_slice(
            br#"
            {
              "creator": "this is a test",
              "modules": {
              }
            }"#,
        )
        .unwrap();
        assert_eq!(result.creator, "this is a test");
        assert_eq!(result.modules.len(), 0);
    }

    #[test]
    fn empty_json_2() {
        let result = Netlist::from_slice(
            br#"
            {
              "modules": {
              }
            }"#,
        )
        .unwrap();
        assert_eq!(result.creator, "");
        assert_eq!(result.modules.len(), 0);
    }

    #[test]
    fn bit_values_test() {
        let result = Netlist::from_slice(
            br#"
            {
              "modules": {
                "mymodule": {
                  "cells": {
                    "mycell": {
                      "type": "celltype",
                      "connections": {
                        "IN": [ "x", 0, "z", 234, "1", "0" ]
                      }
                    }
                  }
                }
              }
            }"#,
        )
        .unwrap();
        assert_eq!(
            result
                .modules
                .get("mymodule")
                .unwrap()
                .cells
                .get("mycell")
                .unwrap()
                .connections
                .get("IN")
                .unwrap(),
            &vec![
                BitVal::S(SpecialBit::X),
                BitVal::N(0),
                BitVal::S(SpecialBit::Z),
                BitVal::N(234),
                BitVal::S(SpecialBit::_1),
                BitVal::S(SpecialBit::_0)
            ]
        );
    }

    #[test]
    #[should_panic]
    fn invalid_bit_value_test() {
        Netlist::from_slice(
            br#"
            {
              "modules": {
                "mymodule": {
                  "cells": {
                    "mycell": {
                      "type": "celltype",
                      "connections": {
                        "IN": [ "w" ]
                      }
                    }
                  }
                }
              }
            }"#,
        )
        .unwrap();
    }

    #[test]
    fn attribute_value_test() {
        let result = Netlist::from_slice(
            br#"
            {
              "modules": {
                "mymodule": {
                  "cells": {
                    "mycell": {
                      "type": "celltype",
                      "parameters": {
                        "testparam": 123
                      },
                      "connections": {}
                    }
                  }
                }
              }
            }"#,
        )
        .unwrap();
        assert_eq!(
            result
                .modules
                .get("mymodule")
                .unwrap()
                .cells
                .get("mycell")
                .unwrap()
                .parameters
                .get("testparam")
                .unwrap(),
            &AttributeVal::N(123)
        );
    }

    #[test]
    #[should_panic]
    fn invalid_attribute_value_test() {
        Netlist::from_slice(
            br#"
            {
              "modules": {
                "mymodule": {
                  "cells": {
                    "mycell": {
                      "type": "celltype",
                      "parameters": {
                        "testparam": [123]
                      },
                      "connections": {}
                    }
                  }
                }
              }
            }"#,
        )
        .unwrap();
    }
}
