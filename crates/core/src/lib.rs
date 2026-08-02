//! The Gandora compiler as a library (GEP-0012).
//!
//! Every language rule — lexing, parsing, macro expansion, resolution,
//! code generation, diagnostics — lives here. The `gan` CLI and the
//! `gandora_core` Python extension are thin consumers.

pub mod ast;
pub mod codegen;
pub mod diag;
pub mod expander;
pub mod jsonc;
pub mod lexer;
pub mod parser;
pub mod printer;
pub mod project;
