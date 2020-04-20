#[macro_use]
extern crate log;

mod c_interface;
mod mdnotes;
mod runtime;
mod warp_fs;

pub use c_interface::*;
pub use runtime::*;

type MdNotesError = String;
