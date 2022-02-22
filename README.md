[![Crates.io](https://img.shields.io/crates/v/yosys-netlist-json.svg)](https://crates.io/crates/yosys-netlist-json)
[![Docs.rs](https://img.shields.io/badge/docs.rs-yosys--netlist--json-informational.svg)](https://docs.rs/yosys-netlist-json)

# Yosys JSON netlist serde structures

Read/write [Yosys](https://github.com/YosysHQ/yosys) JSON files in Rust using serde.

## Reading

```rust
// from a byte slice
let result = Netlist::from_slice(...).unwrap();

// from an io::Read
let result = Netlist::from_reader(reader).unwrap();
```

## Writing

```rust
let mut netlist = Netlist::new("Super cool HDL tool");
...
let json = netlist.to_string().unwrap();
```
