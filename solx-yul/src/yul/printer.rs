//!
//! Printers for all Yul AST node types
//!

use crate::util::printer::print_list_comma_separated;
use crate::util::printer::IPrinter;
use crate::yul::parser::statement::assignment::Assignment;
use crate::yul::parser::statement::block::Block;
use crate::yul::parser::statement::code::Code;
use crate::yul::parser::statement::expression::function_call::name::Name;
use crate::yul::parser::statement::expression::function_call::FunctionCall;
use crate::yul::parser::statement::expression::literal::Literal;
use crate::yul::parser::statement::expression::Expression;
use crate::yul::parser::statement::for_loop::ForLoop;
use crate::yul::parser::statement::function_definition::FunctionDefinition;
use crate::yul::parser::statement::if_conditional::IfConditional;
use crate::yul::parser::statement::object::Object;
use crate::yul::parser::statement::switch::Switch;
use crate::yul::parser::statement::variable_declaration::VariableDeclaration;
use crate::yul::parser::statement::Statement;
use crate::yul::visitor::Visitor;

use super::parser::dialect::Dialect;

impl<T, P> Visitor<P> for T
where
    T: IPrinter,
    P: Dialect,
{
    fn visit_object(&mut self, obj: &Object<P>) {
        let identifier = obj.identifier.as_str();
        self.println(format!("object \"{identifier}\" {{").as_str())
            .unwrap();
        self.increase_indent().unwrap();
        self.visit_code(&obj.code);
        self.println("").unwrap();
        if let Some(inner) = &obj.inner_object {
            self.visit_object(inner)
        }
        self.println("}").unwrap();
        self.decrease_indent().unwrap();
    }

    fn visit_code(&mut self, code: &Code<P>) {
        self.print("code ").unwrap();
        self.visit_block(&code.block);
    }

    fn visit_switch(&mut self, s: &Switch<P>) {
        self.print("switch ").unwrap();
        <T as Visitor<P>>::visit_expression(self, &s.expression);
        self.println("").unwrap();
        for clause in s.cases.iter() {
            self.print("case ").unwrap();
            <T as Visitor<P>>::visit_literal(self, &clause.literal);
            self.print("   ").unwrap();
            self.visit_block(&clause.block);
            self.println("").unwrap();
        }
        if let Some(block) = &s.default {
            self.print("default ").unwrap();
            self.visit_block(block);
            self.println("").unwrap();
        }
    }

    fn visit_for_loop(&mut self, def: &ForLoop<P>) {
        self.print("for ").unwrap();
        self.visit_block(&def.initializer);
        <T as Visitor<P>>::visit_expression(self, &def.condition);
        self.visit_block(&def.finalizer);
        self.println("").unwrap();
        self.visit_block(&def.body);
        self.println("").unwrap();
    }

    fn visit_variable_declaration(&mut self, def: &VariableDeclaration) {
        self.print("let ").unwrap();
        print_list_comma_separated(def.bindings.iter().map(|b| b.inner.as_str()), self).unwrap();
        if let Some(expr) = &def.expression {
            self.print(" := ").unwrap();
            <T as Visitor<P>>::visit_expression(self, expr);
        }
    }

    fn visit_function_definition(&mut self, def: &FunctionDefinition<P>) {
        let identifier: &str = def.identifier.as_str();
        self.print(format!("function {identifier}(").as_str())
            .unwrap();
        let arguments = def.arguments.iter().map(|a| a.inner.as_str());
        print_list_comma_separated(arguments, self).unwrap();
        self.print(") -> ").unwrap();
        let result_identifiers = def.result.iter().map(|r| r.inner.as_str());
        print_list_comma_separated(result_identifiers, self).unwrap();
        self.print(" ").unwrap();
        self.visit_block(&def.body);
        self.println("").unwrap();
    }

    fn visit_name(&mut self, name: &Name) {
        self.print(&name_identifier(name)).unwrap();
    }

    fn visit_function_call(&mut self, call: &FunctionCall) {
        <T as Visitor<P>>::visit_name(self, &call.name);
        self.print("(").unwrap();
        for (idx, a) in call.arguments.iter().enumerate() {
            if idx > 0 {
                self.print(", ").unwrap();
            }
            <T as Visitor<P>>::visit_expression(self, a);
        }
        self.print(")").unwrap();
    }

    fn visit_if_conditional(&mut self, if_conditional: &IfConditional<P>) {
        self.print("if ").unwrap();
        <T as Visitor<P>>::visit_expression(self, &if_conditional.condition);
        self.print(" ").unwrap();
        self.visit_block(&if_conditional.block);
        self.println("").unwrap();
    }

    fn visit_literal(&mut self, lit: &Literal) {
        let inner = &lit.inner;
        if let super::lexer::token::lexeme::literal::Literal::String(_) = inner {
            self.print(format!("\"{inner}\"").as_str()).unwrap();
        } else {
            self.print(format!("{inner}").as_str()).unwrap();
        }
    }

    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::FunctionCall(fc) => <T as Visitor<P>>::visit_function_call(self, fc),
            Expression::Identifier(i) => self.print(i.inner.as_str()).unwrap(),
            Expression::Literal(l) => <T as Visitor<P>>::visit_literal(self, l),
        }
    }
    fn visit_assignment(&mut self, assignment: &Assignment) {
        for (idx, a) in assignment.bindings.iter().enumerate() {
            if idx > 0 {
                self.print(", ").unwrap()
            }
            self.print(a.inner.as_str()).unwrap();
        }
        self.print(" := ").unwrap();
        <T as Visitor<P>>::visit_expression(self, &assignment.initializer);
    }

    fn visit_statement(&mut self, stmt: &Statement<P>) {
        match stmt {
            Statement::Object(o) => self.visit_object(o),
            Statement::Code(c) => self.visit_code(c),
            Statement::Block(b) => self.visit_block(b),
            Statement::Expression(e) => <T as Visitor<P>>::visit_expression(self, e),
            Statement::FunctionDefinition(fd) => self.visit_function_definition(fd),
            Statement::VariableDeclaration(vd) => {
                <T as Visitor<P>>::visit_variable_declaration(self, vd)
            }
            Statement::Assignment(a) => <T as Visitor<P>>::visit_assignment(self, a),
            Statement::IfConditional(i) => <T as Visitor<P>>::visit_if_conditional(self, i),
            Statement::Switch(s) => <T as Visitor<P>>::visit_switch(self, s),
            Statement::ForLoop(f) => <T as Visitor<P>>::visit_for_loop(self, f),
            Statement::Continue(_) => self.print("continue").unwrap(),
            Statement::Break(_) => self.print("break").unwrap(),
            Statement::Leave(_) => self.print("leave").unwrap(),
        }
    }

    fn visit_block(&mut self, block: &Block<P>) {
        if block.statements.is_empty() {
            self.print(" { }").unwrap();
            return;
        }

        if block.statements.len() == 1 {
            self.print("{ ").unwrap();
            self.visit_statement(block.statements.first().unwrap());
            self.print(" }").unwrap();
            return;
        }
        self.println(" {").unwrap();
        self.increase_indent().unwrap();
        for s in block.statements.iter() {
            self.visit_statement(s);
            self.println("").unwrap();
        }
        self.println(" }").unwrap();
        self.decrease_indent().unwrap();
    }
}

/// Shows how an instance of [`Name`] is displayed in Yul code.
pub fn name_identifier(name: &Name) -> String {
    if let Name::Verbatim {
        input_size,
        output_size,
    } = name
    {
        format!("verbatim_{input_size}i_{output_size}o")
    } else if let Name::UserDefined(inner) = name {
        inner.to_string()
    } else {
        String::from(match name {
            Name::Add => "add",
            Name::Sub => "sub",
            Name::Mul => "mul",
            Name::Div => "div",
            Name::Mod => "mod",
            Name::Sdiv => "sdiv",
            Name::Smod => "smod",

            Name::Lt => "lt",
            Name::Gt => "gt",
            Name::Eq => "eq",
            Name::IsZero => "iszero",
            Name::Slt => "slt",
            Name::Sgt => "sgt",

            Name::Or => "or",
            Name::Xor => "xor",
            Name::Not => "not",
            Name::And => "and",
            Name::Shl => "shl",
            Name::Shr => "shr",
            Name::Sar => "sar",
            Name::Byte => "byte",
            Name::Clz => "clz",
            Name::Pop => "pop",

            Name::AddMod => "addmod",
            Name::MulMod => "mulmod",
            Name::Exp => "exp",
            Name::SignExtend => "signextend",

            Name::Keccak256 => "keccak256",

            Name::MLoad => "mload",
            Name::MStore => "mstore",
            Name::MStore8 => "mstore8",
            Name::MCopy => "mcopy",

            Name::SLoad => "sload",
            Name::SStore => "sstore",
            Name::TLoad => "tload",
            Name::TStore => "tstore",
            Name::LoadImmutable => "loadimmutable",
            Name::SetImmutable => "setimmutable",

            Name::CallDataLoad => "calldataload",
            Name::CallDataSize => "calldatasize",
            Name::CallDataCopy => "calldatacopy",
            Name::CodeSize => "codesize",
            Name::CodeCopy => "codecopy",
            Name::ReturnDataSize => "returndatasize",
            Name::ReturnDataCopy => "returndatacopy",
            Name::ExtCodeSize => "extcodesize",
            Name::ExtCodeHash => "extcodehash",

            Name::Return => "return",
            Name::Revert => "revert",

            Name::Log0 => "log0",
            Name::Log1 => "log1",
            Name::Log2 => "log2",
            Name::Log3 => "log3",
            Name::Log4 => "log4",

            Name::Call => "call",
            Name::DelegateCall => "delegatecall",
            Name::StaticCall => "staticcall",

            Name::Create => "create",
            Name::Create2 => "create2",
            Name::DataSize => "datasize",
            Name::DataOffset => "dataoffset",
            Name::DataCopy => "datacopy",

            Name::Stop => "stop",
            Name::Invalid => "invalid",

            Name::LinkerSymbol => "linkersymbol",
            Name::MemoryGuard => "memoryguard",

            Name::Address => "address",
            Name::Caller => "caller",

            Name::CallValue => "callvalue",
            Name::Gas => "gas",
            Name::Balance => "balance",
            Name::SelfBalance => "selfbalance",

            Name::GasLimit => "gaslimit",
            Name::GasPrice => "gasprice",
            Name::Origin => "origin",
            Name::ChainId => "chainid",
            Name::Timestamp => "timestamp",
            Name::Number => "number",
            Name::BlockHash => "blockhash",
            Name::BlobHash => "blobhash",
            Name::Difficulty => "difficulty",
            Name::Prevrandao => "prevrandao",
            Name::CoinBase => "coinbase",
            Name::BaseFee => "basefee",
            Name::BlobBaseFee => "blobbasefee",
            Name::MSize => "msize",

            Name::UnsafeAsm => "unsafeasm",

            Name::CallCode => "callcode",
            Name::Pc => "pc",
            Name::ExtCodeCopy => "extcodecopy",
            Name::SelfDestruct => "selfdestruct",

            Name::UserDefined(_) | Name::Verbatim { .. } => {
                unreachable!()
            }
        })
    }
}
