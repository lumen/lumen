use super::*;

use proptest::strategy::{BoxedStrategy, Just, Strategy};
use proptest::test_runner::{Config, TestRunner};
use proptest::{prop_assert, prop_assert_eq};

use liblumen_alloc::erts::term::{make_pid, next_pid, SmallInteger};

use crate::otp::erlang;
use crate::process::{self, SchedulerDependentAlloc};
use crate::scheduler::{with_process, with_process_arc};
use crate::test::{
    has_heap_message, has_message, has_process_message, registered_name, strategy, timeout_message,
};

mod register_2;
mod registered_0;
mod rem_2;
mod send_3;
mod send_after_3;
mod send_after_4;
mod setelement_3;
mod size_1;
mod split_binary_2;
mod start_timer_3;
mod start_timer_4;
mod subtract_list_2;
mod throw_1;
mod tl_1;
mod tuple_size_1;
mod tuple_to_list_1;
mod unregister_1;
mod whereis_1;
mod xor_2;

fn errors_badarith<F>(actual: F)
where
    F: FnOnce(&Process) -> Result,
{
    with_process(|process| assert_badarith!(actual(&process)))
}
