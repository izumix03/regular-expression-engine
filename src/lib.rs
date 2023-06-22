//! # 正規表現エンジン用クレート
//!
//! ## 利用例
//!
//! ```
//! use regex;
//! let expr = "a(bc)+|c(def)*";
//! let line = "cdefdefdef";
//! regex::do_matching(expr, line, true);
//! regex::print(expr);
//! ```
mod engine;
mod helpers;

pub use engine::{do_matching, print};