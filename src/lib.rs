//! 正規表現エンジン用クレート
//!
//! ## Example
//! ```
//! use regex_engine;
//! let expr = "a(bc)+|c(def)*";
//! let line = "cdefdefdef";
//! regex_engine::do_matching(expr, line, true);
//! regex_engine::print(expr);
//! ```

mod engine;
mod helper;

pub use engine::{do_matching, print};
