mod adapters;
mod config;
mod layout;
mod model;
mod snapshot;
mod usecases;
mod util;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use layout::{
    DoctorReport, InitResult, ProjectLayout, ToolStatus, doctor, init_project, resolve_root,
};
pub use model::*;
pub use snapshot::WorkItemSnapshot;
pub use usecases::*;

pub(crate) use config::*;
pub(crate) use layout::*;
pub(crate) use util::*;

#[cfg(test)]
mod tests;
