test_stdout!(without_number_addend_errors_badarith, "{caught, error, badarith}\n{caught, error, badarith}\n{caught, error, badarith}\n{caught, error, badarith}\n{caught, error, badarith}\n{caught, error, badarith}\n{caught, error, badarith}\n{caught, error, badarith}\n{caught, error, badarith}\n");

test_stdout!(
    with_big_integer_addend_returns_big_integer,
    "true\ntrue\ntrue\n"
);
test_stdout!(with_float_addend_with_overflow_returns_max_float, "179769313486231570000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0\n");
test_stdout!(with_float_addend_with_underflow_returns_min_float, "-179769313486231570000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0\n");
test_stdout!(
    with_float_addend_without_underflow_or_overflow_returns_float,
    "true\n"
);
test_stdout!(
    with_small_integer_addend_with_overflow_returns_big_integer,
    "true\ntrue\ntrue\n"
);
test_stdout!(
    with_small_integer_addend_with_underflow_returns_big_integer,
    "true\ntrue\ntrue\n"
);
test_stdout!(
    with_small_integer_addend_without_underflow_or_overflow_returns_small_integer,
    "true\ntrue\ntrue\n"
);
