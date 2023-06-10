#![warn(missing_docs)]

//! Simple crate for SPAYD (Short Payment Descriptor) generation
//! # Example
//! ```
//! let spayd = Spayd::builder()
//!     .account("CZ7907000000001234567890".to_string())
//!     .amount("239.50".to_string())
//!     .build();
//! 
//! let result = spayd.spayd_string().unwrap();
//! 
//! // "SPD*1.0*ACC:CZ7907000000001234567890*AM:239.50"
//! ```
//! 
//! # TODO
//! - [x] SPAYD string generation
//! - [ ] QR code generation as an optional feature

mod spayd;
pub use spayd::*;
