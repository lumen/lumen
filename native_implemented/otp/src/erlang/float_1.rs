use std::convert::TryInto;

use anyhow::*;

use liblumen_alloc::erts::exception;
use liblumen_alloc::erts::process::Process;
use liblumen_alloc::erts::term::prelude::*;

#[native_implemented::function(erlang:float/1)]
pub fn result(process: &Process, number: Term) -> exception::Result<Term> {
    if number.is_boxed_float() {
        Ok(number)
    } else {
        let f: f64 = number
            .try_into()
            .with_context(|| term_is_not_number!(number))?;

        Ok(process.float(f))
    }
}
