#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::cast_precision_loss,
    clippy::too_many_arguments,
    clippy::return_self_not_must_use,
    clippy::similar_names
)]

pub mod cli;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod sema;
pub mod ui;
