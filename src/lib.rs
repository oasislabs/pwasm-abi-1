//! WASM ABI Tools

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]
#![warn(missing_docs)]
#![cfg_attr(feature = "strict", deny(unused))]

extern crate byteorder;
extern crate owasm_std;
extern crate uint;

#[cfg(test)]
#[macro_use]
extern crate hex_literal;

#[cfg(not(feature = "std"))]
#[allow(unused)]
#[macro_use]
extern crate alloc;

pub mod eth;

/// Custom types which AbiType supports
pub mod types {
    pub use owasm_std::{hash::*, Vec};
    pub use uint::U256;
}

mod lib {

    mod core {
        #[cfg(not(feature = "std"))]
        pub use core::*;
        #[cfg(feature = "std")]
        pub use std::*;
    }

    pub use self::core::{
        cmp, i16, i32, i64, i8, isize, iter, mem, ops, slice, str, u16, u32, u64, u8, usize,
    };

    pub use self::core::{
        cell::{Cell, RefCell},
        clone::{self, Clone},
        convert::{self, From, Into},
        default::{self, Default},
        fmt::{self, Debug, Display},
        marker::{self, PhantomData},
        option::{self, Option},
        result::{self, Result},
    };

    #[cfg(not(feature = "std"))]
    pub use alloc::borrow::{Cow, ToOwned};
    #[cfg(feature = "std")]
    pub use std::borrow::{Cow, ToOwned};

    #[cfg(not(feature = "std"))]
    pub use alloc::string::{String, ToString};
    #[cfg(feature = "std")]
    pub use std::string::String;

    #[cfg(not(feature = "std"))]
    pub use alloc::vec::Vec;
    #[cfg(feature = "std")]
    pub use std::vec::Vec;

    #[cfg(not(feature = "std"))]
    pub use alloc::boxed::Box;
    #[cfg(feature = "std")]
    pub use std::boxed::Box;
}
