#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic
)]

pub mod build;
pub mod clean;
pub mod convert;
pub mod init;
pub mod install;
pub mod uninstall;
