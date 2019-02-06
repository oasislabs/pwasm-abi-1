//! WASM ABI Tools

#![warn(missing_docs)]
#![cfg_attr(feature="strict", deny(unused))]

extern crate byteorder;

#[cfg(test)]
#[cfg_attr(all(test), macro_use)]
extern crate hex_literal;

pub mod eth;
