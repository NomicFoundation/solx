//!
//! Tests for the Yul pretty printer.
//!

#![cfg(test)]

use solx_yul::util::printer::write_printer::WritePrinter;
use solx_yul::yul::lexer::Lexer;
use solx_yul::yul::parser::statement::Statement;
use solx_yul::yul::parser::statement::expression::Expression;
use solx_yul::yul::visitor::Visitor;

fn print_expression(input: &str) -> String {
    let mut lexer = Lexer::new(input);
    let expression = Expression::parse(&mut lexer, None).expect("Always valid");
    let mut result = String::new();
    let mut writer = WritePrinter::<&mut String>::new(&mut result);
    Visitor::visit_expression(&mut writer, &expression);
    result
}

fn print_statement(input: &str) -> String {
    let mut lexer = Lexer::new(input);
    let statement = Statement::parse(&mut lexer, None).unwrap().0;
    let mut result = String::new();
    let mut writer = WritePrinter::<&mut String>::new(&mut result);
    Visitor::visit_statement(&mut writer, &statement);
    result
}

mod expressions {
    use crate::print_expression;

    #[test]
    fn test_literal() {
        assert_eq!(print_expression("123"), "123");
        assert_eq!(print_expression(" true "), "true");
        assert_eq!(print_expression(" false "), "false");
        assert_eq!(print_expression(" \"hello\""), "\"hello\"");
    }

    #[test]
    fn test_identifier() {
        assert_eq!(print_expression(" x "), "x");
        assert_eq!(print_expression(" x_$1231 "), "x_$1231");
    }

    #[test]
    fn test_function_call() {
        assert_eq!(print_expression(" f() "), "f()");
        assert_eq!(print_expression(" sub(x,y) "), "sub(x, y)");
        assert_eq!(
            print_expression(" add(sub(x,y), 1,2,3) "),
            "add(sub(x, y), 1, 2, 3)"
        );
    }
}

mod statements {
    use crate::print_statement;

    #[test]
    fn statement_for() {
        let expected = "object \"test\" {\n  code  { }\n  object \"test_deployed\" {\n    code { function power(base, exponent) -> result  {\n      result := 1\n      for { let i := 0 }lt(i, exponent){ i := add(i, 1) }\n       {\n        result := mul(result, base)\n        break\n        continue\n         }\n      \n      \n       }\n    \n     }\n    }\n  }\n";
        assert_eq!(
            print_statement(
                r#"
object "test" {
    code { }
    object "test_deployed" {
        code {
function power(base, exponent) -> result
    {
        result := 1
        for { let i := 0 } lt(i, exponent) { i := add(i, 1) }
        {
            result := mul(result, base)
break
continue
        }
    }
       }
    }

}
"#
            ),
            expected
        );
    }

    #[test]
    fn test_let() {
        let expected = "object \"ecadd\" {\n  code  { }\n  object \"ecadd_deployed\" {\n    code  {\n      let x\n      let a := 4\n       }\n    \n    }\n  }\n";
        assert_eq!(
            print_statement(
                r#"
object "ecadd" {
    code { }
    object "ecadd_deployed" {
        code {
        let x
        let a := 4
}
    }

}
"#
            ),
            expected
        );
    }

    #[test]
    fn test_assignment() {
        let expected = "object \"ecadd\" {\n  code  { }\n  object \"ecadd_deployed\" {\n    code  {\n      let x\n      x := 4\n       }\n    \n    }\n  }\n";
        assert_eq!(
            print_statement(
                r#"
object "ecadd" {
    code { }
    object "ecadd_deployed" {
        code {
        let x
        x := 4
}
    }

}
"#
            ),
            expected
        );
    }

    #[test]
    fn test_if() {
        let expected = "object \"ecadd\" {\n  code  { }\n  object \"ecadd_deployed\" {\n    code { if lt(a, b) { sstore(0, 1) }\n     }\n    }\n  }\n";
        assert_eq!(
            print_statement(
                r#"
object "ecadd" {
    code { }
    object "ecadd_deployed" {
        code {

if lt(a, b) { sstore(0, 1) }

}
    }

}
"#
            ),
            expected
        );
    }

    #[test]
    fn test_switch() {
        let expected = "object \"ecadd\" {\n  code  { }\n  object \"ecadd_deployed\" {\n    code {  {\n      let x := 0\n      switch calldataload(4)\n      case 0   { x := calldataload(0x24) }\n      default { x := calldataload(0x44) }\n      \n      sstore(0, div(x, 2))\n       }\n     }\n    }\n  }\n";
        assert_eq!(
            print_statement(
                r#"
object "ecadd" {
    code { }
    object "ecadd_deployed" {
        code {

{
    let x := 0
    switch calldataload(4)
    case 0 {
        x := calldataload(0x24)
    }
    default {
        x := calldataload(0x44)
    }
    sstore(0, div(x, 2))
}

}
    }

}
"#
            ),
            expected
        );
    }
}
