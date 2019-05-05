use super::*;

#[test]
fn with_lesser_small_integer_right_returns_false() {
    is_equal_or_less_than(|_, process| (-1).into_process(&process), false)
}

#[test]
fn with_greater_small_integer_right_returns_true() {
    is_equal_or_less_than(|_, process| 1.into_process(&process), true)
}

#[test]
fn with_lesser_big_integer_right_returns_false() {
    is_equal_or_less_than(
        |_, process| (crate::integer::small::MIN - 1).into_process(&process),
        false,
    )
}

#[test]
fn with_greater_big_integer_right_returns_true() {
    is_equal_or_less_than(
        |_, process| (crate::integer::small::MAX + 1).into_process(&process),
        true,
    )
}

#[test]
fn with_lesser_float_right_returns_false() {
    is_equal_or_less_than(|_, process| (-1.0).into_process(&process), false)
}

#[test]
fn with_greater_float_right_returns_true() {
    is_equal_or_less_than(|_, process| 1.0.into_process(&process), true)
}

#[test]
fn with_atom_right_returns_true() {
    is_equal_or_less_than(|_, _| Term::str_to_atom("right", DoNotCare).unwrap(), true);
}

#[test]
fn with_local_reference_right_returns_true() {
    is_equal_or_less_than(|_, process| Term::next_local_reference(process), true);
}

#[test]
fn with_local_pid_right_returns_true() {
    is_equal_or_less_than(|_, _| Term::local_pid(0, 1).unwrap(), true);
}

#[test]
fn with_external_pid_right_returns_true() {
    is_equal_or_less_than(
        |_, process| Term::external_pid(1, 2, 3, &process).unwrap(),
        true,
    );
}

#[test]
fn with_tuple_right_returns_true() {
    is_equal_or_less_than(|_, process| Term::slice_to_tuple(&[], &process), true);
}

#[test]
fn with_map_right_returns_true() {
    is_equal_or_less_than(|_, process| Term::slice_to_map(&[], &process), true);
}

#[test]
fn with_empty_list_right_returns_true() {
    is_equal_or_less_than(|_, _| Term::EMPTY_LIST, true);
}

#[test]
fn with_list_right_returns_true() {
    is_equal_or_less_than(
        |_, process| Term::cons(0.into_process(&process), 1.into_process(&process), &process),
        true,
    );
}

#[test]
fn with_heap_binary_right_returns_true() {
    is_equal_or_less_than(|_, process| Term::slice_to_binary(&[], &process), true);
}

#[test]
fn with_subbinary_right_returns_true() {
    is_equal_or_less_than(|_, process| bitstring!(1 :: 1, &process), true);
}

fn is_equal_or_less_than<R>(right: R, expected: bool)
where
    R: FnOnce(Term, &Process) -> Term,
{
    super::is_equal_or_less_than(|process| 0.0.into_process(&process), right, expected);
}
