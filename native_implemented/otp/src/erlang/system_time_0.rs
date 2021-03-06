#[cfg(test)]
mod test;

use liblumen_alloc::erts::process::Process;
use liblumen_alloc::erts::term::prelude::*;

use crate::runtime::time::{system, Unit::Native};

#[native_implemented::function(erlang:system_time/0)]
pub fn result(process: &Process) -> Term {
    let big_int = system::time_in_unit(Native);

    process.integer(big_int)
}
