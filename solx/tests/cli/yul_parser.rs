//!
//! CLI tests for the Yul parser coverage.
//!

use predicates::prelude::*;
use test_case::test_case;

#[test]
fn parser_coverage_bin() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/ParserCoverage.yul"),
        "--yul",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

#[test]
fn parser_coverage_standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("yul_parser_coverage.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"))
        .stdout(predicate::str::contains("object"));

    Ok(())
}

#[test]
fn syntax_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/SyntaxError.yul"),
        "--yul",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains("Error"));

    Ok(())
}

#[test_case(
    crate::common::contract!("yul/ErrorBlockInvalidToken.yul"),
    "InvalidToken",
    r#"found: "("#;
    "block_invalid_token"
)]
#[test_case(
    crate::common::contract!("yul/ErrorBlockMissingBrace.yul"),
    "InvalidToken",
    r#"expected: ["{"]"#;
    "block_missing_brace"
)]
#[test_case(
    crate::common::contract!("yul/ErrorExpressionInvalid.yul"),
    "InvalidToken",
    r#"found: ":="#;
    "expression_invalid"
)]
#[test_case(
    crate::common::contract!("yul/ErrorReservedLetBinding.yul"),
    "ReservedIdentifier",
    "basefee";
    "reserved_let_binding"
)]
#[test_case(
    crate::common::contract!("yul/ErrorFunctionLiteralName.yul"),
    "InvalidToken",
    r#"found: "256"#;
    "function_literal_name"
)]
#[test_case(
    crate::common::contract!("yul/ErrorFunctionMissingParen.yul"),
    "InvalidToken",
    r#"expected: ["("]"#;
    "function_missing_paren"
)]
#[test_case(
    crate::common::contract!("yul/ErrorFunctionMissingCloseParen.yul"),
    "InvalidToken",
    r#"expected: [")"]"#;
    "function_missing_close_paren"
)]
#[test_case(
    crate::common::contract!("yul/ErrorFunctionBadReturn.yul"),
    "InvalidToken",
    r#"expected: ["->", "{"]"#;
    "function_bad_return"
)]
#[test_case(
    crate::common::contract!("yul/ErrorFunctionReservedName.yul"),
    "ReservedIdentifier",
    "basefee";
    "function_reserved_name"
)]
#[test_case(
    crate::common::contract!("yul/ErrorFunctionInvalidAttrs.yul"),
    "InvalidAttributes",
    "UnknownAttribute";
    "function_invalid_attrs"
)]
#[test_case(
    crate::common::contract!("yul/ErrorFunctionInvalidAttrsRepeated.yul"),
    "InvalidAttributes",
    "UnknownAttribute2";
    "function_invalid_attrs_repeated"
)]
#[test_case(
    crate::common::contract!("yul/ErrorSwitchNoCaseDefault.yul"),
    "InvalidToken",
    r#"found: "branch"#;
    "switch_no_case_default"
)]
#[test_case(
    crate::common::contract!("yul/ErrorSwitchCaseNonLiteral.yul"),
    "InvalidToken",
    r#"found: "x"#;
    "switch_case_non_literal"
)]
#[test_case(
    crate::common::contract!("yul/ErrorBlockBareSymbol.yul"),
    "InvalidToken",
    r#"found: ":="#;
    "block_bare_symbol"
)]
#[test_case(
    crate::common::contract!("yul/ErrorIfMissingBlock.yul"),
    "InvalidToken",
    r#"expected: ["{"]"#;
    "if_missing_block"
)]
#[test_case(
    crate::common::contract!("yul/ErrorAssignmentMultiMissing.yul"),
    "InvalidToken",
    r#"expected: [":="]"#;
    "assignment_multi_missing"
)]
#[test_case(
    crate::common::contract!("yul/ErrorStatementKeywordCase.yul"),
    "InvalidToken",
    r#"found: "case""#;
    "statement_keyword_case"
)]
fn parser_error(path: &str, error_type: &str, error_context: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[path, "--yul", "--bin"];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains(error_type))
        .stderr(predicate::str::contains(error_context));

    Ok(())
}

#[test]
fn unsupported_callcode() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/ErrorUnsupportedCallcode.yul"),
        "--yul",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(
        predicate::str::contains("CALLCODE").and(predicate::str::contains("not supported")),
    );

    Ok(())
}

#[test]
fn unsupported_pc() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/ErrorUnsupportedPc.yul"),
        "--yul",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains("PC").and(predicate::str::contains("not supported")));

    Ok(())
}

#[test]
fn unsupported_selfdestruct() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/ErrorUnsupportedSelfdestruct.yul"),
        "--yul",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(
        predicate::str::contains("SELFDESTRUCT").and(predicate::str::contains("not supported")),
    );

    Ok(())
}
