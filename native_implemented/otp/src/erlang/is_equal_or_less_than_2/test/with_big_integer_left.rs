use super::*;

#[test]
fn with_lesser_small_integer_right_returns_false() {
    is_equal_or_less_than(|_, process| process.integer(0), false)
}

#[test]
fn with_greater_small_integer_right_returns_true() {
    super::is_equal_or_less_than(
        |process| process.integer(SmallInteger::MIN_VALUE - 1),
        |_, process| process.integer(SmallInteger::MIN_VALUE),
        true,
    );
}

#[test]
fn with_lesser_big_integer_right_returns_false() {
    is_equal_or_less_than(
        |_, process| process.integer(SmallInteger::MIN_VALUE - 1),
        false,
    )
}

#[test]
fn with_same_value_big_integer_right_returns_true() {
    is_equal_or_less_than(
        |_, process| process.integer(SmallInteger::MAX_VALUE + 1),
        true,
    )
}

#[test]
fn with_greater_big_integer_right_returns_true() {
    is_equal_or_less_than(
        |_, process| process.integer(SmallInteger::MAX_VALUE + 2),
        true,
    )
}

#[test]
fn with_lesser_float_right_returns_false() {
    is_equal_or_less_than(|_, process| process.float(1.0), false)
}

#[test]
fn with_greater_float_right_returns_true() {
    super::is_equal_or_less_than(
        |process| process.integer(SmallInteger::MIN_VALUE - 1),
        |_, process| process.float(1.0),
        true,
    );
}

#[test]
fn without_number_returns_true() {
    run!(
        |arc_process| {
            (
                strategy::term::integer::big(arc_process.clone()),
                strategy::term::is_not_number(arc_process.clone()),
            )
        },
        |(left, right)| {
            prop_assert_eq!(result(left, right), true.into());

            Ok(())
        },
    );
}

fn is_equal_or_less_than<R>(right: R, expected: bool)
where
    R: FnOnce(Term, &Process) -> Term,
{
    super::is_equal_or_less_than(
        |process| process.integer(SmallInteger::MAX_VALUE + 1),
        right,
        expected,
    );
}
