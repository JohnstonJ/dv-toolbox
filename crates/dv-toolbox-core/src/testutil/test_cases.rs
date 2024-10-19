use display_error_chain::ErrorChainExt;
use garde::Validate;
use googletest::prelude::*;
use serde::{Deserialize, Serialize};
use serde_test::Token;

#[derive(Debug)]
pub(crate) struct ValidateFailureTestCase<'a, T, C>
where
    T: Validate<Context = C>,
{
    pub(crate) value: T,
    pub(crate) err: &'a str,
    pub(crate) ctx: C,
}

/// Test validation failures of a value
pub(crate) fn run_validate_failure_test_case<T, C>(tc: &ValidateFailureTestCase<T, C>)
where
    T: Validate<Context = C>,
{
    let result = tc.value.validate_with(&tc.ctx);
    expect_that!(result.map_err(|e| e.chain().to_string()), err(eq(tc.err)));
}

#[derive(Debug)]
pub(crate) struct SerDeTestCase<'a, T>
where
    T: Serialize + Deserialize<'a> + PartialEq + std::fmt::Debug,
{
    pub(crate) value: T,
    pub(crate) tokens: &'a [Token],
}

#[derive(Debug)]
pub(crate) struct DeserializeErrorTestCase<'a> {
    pub(crate) tokens: &'a [Token],
    pub(crate) err: &'a str,
}
