use super::*;

use num_traits::Num;

#[test]
fn with_atom_shift_errors_badarith() {
    with_shift_errors_badarith(|_| Term::str_to_atom("shift", DoNotCare).unwrap());
}

#[test]
fn with_local_reference_shift_errors_badarith() {
    with_shift_errors_badarith(|mut process| Term::local_reference(&mut process));
}

#[test]
fn with_empty_list_shift_errors_badarith() {
    with_shift_errors_badarith(|_| Term::EMPTY_LIST);
}

#[test]
fn with_list_shift_errors_badarith() {
    with_shift_errors_badarith(|mut process| {
        Term::cons(
            0.into_process(&mut process),
            1.into_process(&mut process),
            &mut process,
        )
    });
}

#[test]
fn with_negative_with_overflow_shifts_left_and_returns_big_integer() {
    with(|integer, mut process| {
        let shift = (-64).into_process(&mut process);

        assert_eq!(
            erlang::bsr_2(integer, shift, &mut process),
            Ok(<BigInt as Num>::from_str_radix(
                "100000000000000000000000000000000000000000000000000000000000000000",
                2
            )
            .unwrap()
            .into_process(&mut process))
        );
    });
}

#[test]
fn with_negative_without_overflow_shifts_left_and_returns_small_integer() {
    with(|integer, mut process| {
        let shift = (-1).into_process(&mut process);

        assert_eq!(
            erlang::bsr_2(integer, shift, &mut process),
            Ok(0b100.into_process(&mut process))
        );
    });
}

#[test]
fn with_zero_returns_same_small_integer() {
    with(|integer, mut process| {
        assert_eq!(
            erlang::bsr_2(integer, 0.into_process(&mut process), &mut process),
            Ok(integer)
        );
    });
}

#[test]
fn with_positive_without_underflow_returns_small_integer() {
    with(|integer, mut process| {
        let shift = 1.into_process(&mut process);

        let result = erlang::bsr_2(integer, shift, &mut process);

        assert!(result.is_ok());

        let shifted = result.unwrap();

        assert_eq!(shifted.tag(), SmallInteger);
        assert_eq!(shifted, 0b1.into_process(&mut process));
    })
}

#[test]
fn with_positive_with_underflow_returns_zero() {
    with(|integer, mut process| {
        let shift = 3.into_process(&mut process);

        assert_eq!(
            erlang::bsr_2(integer, shift, &mut process),
            Ok(0.into_process(&mut process))
        );
    });
}

#[test]
fn with_float_shift_errors_badarith() {
    with_shift_errors_badarith(|mut process| 1.0.into_process(&mut process));
}

#[test]
fn with_local_pid_shift_errors_badarith() {
    with_shift_errors_badarith(|_| Term::local_pid(0, 1).unwrap());
}

#[test]
fn with_external_pid_shift_errors_badarith() {
    with_shift_errors_badarith(|mut process| Term::external_pid(1, 2, 3, &mut process).unwrap());
}

#[test]
fn with_tuple_shift_errors_badarith() {
    with_shift_errors_badarith(|mut process| Term::slice_to_tuple(&[], &mut process));
}

#[test]
fn with_map_is_shift_errors_badarith() {
    with_shift_errors_badarith(|mut process| Term::slice_to_map(&[], &mut process));
}

#[test]
fn with_heap_binary_shift_errors_badarith() {
    with_shift_errors_badarith(|mut process| Term::slice_to_binary(&[], &mut process));
}

#[test]
fn with_subbinary_shift_errors_badarith() {
    with_shift_errors_badarith(|mut process| bitstring!(1 :: 1, &mut process));
}

fn with<F>(f: F)
where
    F: FnOnce(Term, &mut Process) -> (),
{
    with_process(|mut process| {
        let integer = 0b10.into_process(&mut process);

        f(integer, &mut process)
    })
}

fn with_shift_errors_badarith<S>(shift: S)
where
    S: FnOnce(&mut Process) -> Term,
{
    super::errors_badarith(|mut process| {
        let integer: Term = 0b10.into_process(&mut process);

        assert_eq!(integer.tag(), SmallInteger);

        let shift = shift(&mut process);

        erlang::bsr_2(integer, shift, &mut process)
    });
}
