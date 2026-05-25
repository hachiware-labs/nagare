mod adapters;
mod artifacts;
mod config;
mod dispatch;
mod handoff;
mod layout;
mod model;
mod output_contract;
mod recovery;
mod review;
mod scenario;
mod snapshot;
mod usecases;
mod util;
mod work_items;
mod workflow;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use dispatch::*;
pub use handoff::*;
pub use layout::{
    DoctorReport, InitResult, ProjectLayout, ToolStatus, doctor, init_project, resolve_root,
};
pub use model::*;
pub use recovery::*;
pub use review::*;
pub use scenario::*;
pub use snapshot::{WorkItemSnapshot, WorkItemTimelineEvent};
pub use usecases::*;
pub use work_items::*;
pub use workflow::*;

pub(crate) use artifacts::*;
pub(crate) use config::*;
pub(crate) use layout::*;
pub(crate) use output_contract::*;
pub(crate) use util::*;

#[cfg(test)]
mod tests;
