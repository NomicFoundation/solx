//!
//! Yul expression lowering: literals, path reads, builtin calls, and calls into the block's own Yul
//! functions.
//!

use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::YulExpression;
use slang_solidity_v2::ast::YulFunctionCallExpression;

use solx_mlir::Type as MlirType;
use solx_mlir::Word;
use solx_mlir::YulCmpPredicate;

use crate::scope::assembly::AssemblyScope;

impl<'function, 'contract, 'source_unit, 'context>
    AssemblyScope<'function, 'contract, 'source_unit, 'context>
{
    /// A Yul expression in value position.
    pub fn yul_expression(&mut self, node: &YulExpression) -> Word<'context> {
        let mut words = self.yul_expression_words(node);
        assert_eq!(
            words.len(),
            1,
            "a Yul expression in value position yields one word"
        );
        words.remove(0)
    }

    /// A Yul expression in multi-value position: only a call to a multi-return Yul function yields
    /// more than one word, and only a call yields none.
    pub fn yul_expression_words(&mut self, node: &YulExpression) -> Vec<Word<'context>> {
        match node {
            YulExpression::YulLiteral(literal) => {
                vec![Word::constant(&literal.integer_value(), self)]
            }
            YulExpression::YulPath(path) => {
                let reference = self.yul_reference(path);
                vec![reference.read(self)]
            }
            YulExpression::YulFunctionCallExpression(call) => self.yul_call(call),
        }
    }

    /// A Yul expression emitted for its effects, discarding whatever it yields.
    pub fn yul_expression_effect(&mut self, node: &YulExpression) {
        self.yul_expression_words(node);
    }

    /// A Yul call: a builtin, or a call into a Yul function the enclosing assembly block declares.
    fn yul_call(&mut self, node: &YulFunctionCallExpression) -> Vec<Word<'context>> {
        let YulExpression::YulPath(callee) = node.operand() else {
            unreachable!("a Yul call names its callee by path");
        };
        let name = callee
            .iter()
            .next()
            .expect("a Yul callee path names one identifier");
        // Yul evaluates an argument list right to left - the order the EVM pushes them in. Only the
        // emission order is reversed; the operand list itself stays in source order.
        let expressions: Vec<_> = node.arguments().iter().collect();
        let mut arguments: Vec<_> = expressions
            .iter()
            .rev()
            .map(|argument| self.yul_expression(argument))
            .collect();
        arguments.reverse();

        if let Some(built_in) = name.resolve_to_built_in() {
            return self.yul_builtin(built_in, &arguments);
        }
        let Some(Definition::YulFunction(definition)) = name.resolve_to_definition() else {
            unreachable!(
                "a Yul callee is a builtin or a Yul function: {}",
                name.name()
            );
        };
        let signature = self.signature(definition.node_id());
        let symbol = signature.name.clone();
        let results = vec![MlirType::yul_word(self.melior); signature.results];
        Word::call_function(symbol.as_str(), &arguments, &results, self)
    }

    /// The Yul builtin `built_in` applied to `arguments`. Every builtin takes and yields words, so
    /// the operands are wired positionally; the ones that yield nothing return an empty list.
    fn yul_builtin(
        &mut self,
        built_in: BuiltIn,
        arguments: &[Word<'context>],
    ) -> Vec<Word<'context>> {
        let one = |word| vec![word];
        match built_in {
            BuiltIn::YulAdd => one(Word::add(arguments[0], arguments[1], self)),
            BuiltIn::YulSub => one(Word::sub(arguments[0], arguments[1], self)),
            BuiltIn::YulMul => one(Word::mul(arguments[0], arguments[1], self)),
            BuiltIn::YulDiv => one(Word::div(arguments[0], arguments[1], self)),
            BuiltIn::YulSdiv => one(Word::sdiv(arguments[0], arguments[1], self)),
            BuiltIn::YulMod => one(Word::r#mod(arguments[0], arguments[1], self)),
            BuiltIn::YulSmod => one(Word::smod(arguments[0], arguments[1], self)),
            BuiltIn::YulExp => one(Word::exp(arguments[0], arguments[1], self)),
            BuiltIn::YulAddmod => one(Word::addmod(arguments[0], arguments[1], arguments[2], self)),
            BuiltIn::YulMulmod => one(Word::mulmod(arguments[0], arguments[1], arguments[2], self)),
            BuiltIn::YulSignextend => one(Word::signextend(arguments[0], arguments[1], self)),

            BuiltIn::YulAnd => one(Word::and(arguments[0], arguments[1], self)),
            BuiltIn::YulOr => one(Word::or(arguments[0], arguments[1], self)),
            BuiltIn::YulXor => one(Word::xor(arguments[0], arguments[1], self)),
            BuiltIn::YulNot => one(Word::not(arguments[0], self)),
            BuiltIn::YulShl => one(Word::shl(arguments[0], arguments[1], self)),
            BuiltIn::YulShr => one(Word::shr(arguments[0], arguments[1], self)),
            BuiltIn::YulSar => one(Word::sar(arguments[0], arguments[1], self)),
            BuiltIn::YulClz => one(Word::clz(arguments[0], self)),
            BuiltIn::YulByte => one(Word::byte(arguments[0], arguments[1], self)),

            BuiltIn::YulLt => one(self.yul_compare(arguments, YulCmpPredicate::UnsignedLt)),
            BuiltIn::YulGt => one(self.yul_compare(arguments, YulCmpPredicate::UnsignedGt)),
            BuiltIn::YulSlt => one(self.yul_compare(arguments, YulCmpPredicate::SignedLt)),
            BuiltIn::YulSgt => one(self.yul_compare(arguments, YulCmpPredicate::SignedGt)),
            BuiltIn::YulEq => one(self.yul_compare(arguments, YulCmpPredicate::Eq)),
            BuiltIn::YulIszero => {
                let zero = self.word_zero();
                one(Word::compare(arguments[0], zero, YulCmpPredicate::Eq, self))
            }
            // `pop` discards its operand, which the caller already evaluated for its effects.
            BuiltIn::YulPop => Vec::new(),

            BuiltIn::YulMload => one(Word::mload(arguments[0], self)),
            BuiltIn::YulMstore => {
                Word::mstore(arguments[0], arguments[1], self);
                Vec::new()
            }
            BuiltIn::YulMstore8 => {
                Word::mstore8(arguments[0], arguments[1], self);
                Vec::new()
            }
            BuiltIn::YulMcopy => {
                Word::mcopy(arguments[0], arguments[1], arguments[2], self);
                Vec::new()
            }
            BuiltIn::YulMsize => one(Word::msize(self)),
            BuiltIn::YulKeccak256 => one(Word::keccak256(arguments[0], arguments[1], self)),

            BuiltIn::YulSload => one(Word::sload(arguments[0], self)),
            BuiltIn::YulSstore => {
                Word::sstore(arguments[0], arguments[1], self);
                Vec::new()
            }
            BuiltIn::YulTload => one(Word::tload(arguments[0], self)),
            BuiltIn::YulTstore => {
                Word::tstore(arguments[0], arguments[1], self);
                Vec::new()
            }

            BuiltIn::YulCalldataload => one(Word::calldataload(arguments[0], self)),
            BuiltIn::YulCalldatasize => one(Word::calldatasize(self)),
            BuiltIn::YulCalldatacopy => {
                Word::calldatacopy(arguments[0], arguments[1], arguments[2], self);
                Vec::new()
            }
            BuiltIn::YulReturndatasize => one(Word::returndatasize(self)),
            BuiltIn::YulReturndatacopy => {
                Word::returndatacopy(arguments[0], arguments[1], arguments[2], self);
                Vec::new()
            }
            BuiltIn::YulCodesize => one(Word::codesize(self)),
            BuiltIn::YulCodecopy => {
                Word::codecopy(arguments[0], arguments[1], arguments[2], self);
                Vec::new()
            }
            BuiltIn::YulExtcodesize => one(Word::extcodesize(arguments[0], self)),
            BuiltIn::YulExtcodehash => one(Word::extcodehash(arguments[0], self)),
            BuiltIn::YulExtcodecopy => {
                Word::extcodecopy(arguments[0], arguments[1], arguments[2], arguments[3], self);
                Vec::new()
            }

            BuiltIn::YulCreate => one(Word::create(arguments[0], arguments[1], arguments[2], self)),
            BuiltIn::YulCreate2 => one(Word::create2(
                arguments[0],
                arguments[1],
                arguments[2],
                arguments[3],
                self,
            )),
            BuiltIn::YulCall => one(Word::call(
                arguments[0],
                arguments[1],
                arguments[2],
                arguments[3],
                arguments[4],
                arguments[5],
                arguments[6],
                self,
            )),
            BuiltIn::YulCallcode => one(Word::callcode(
                arguments[0],
                arguments[1],
                arguments[2],
                arguments[3],
                arguments[4],
                arguments[5],
                arguments[6],
                self,
            )),
            BuiltIn::YulStaticcall => one(Word::staticcall(
                arguments[0],
                arguments[1],
                arguments[2],
                arguments[3],
                arguments[4],
                arguments[5],
                self,
            )),
            BuiltIn::YulDelegatecall => one(Word::delegatecall(
                arguments[0],
                arguments[1],
                arguments[2],
                arguments[3],
                arguments[4],
                arguments[5],
                self,
            )),

            BuiltIn::YulReturn => {
                Word::r#return(arguments[0], arguments[1], self);
                Vec::new()
            }
            BuiltIn::YulRevert => {
                Word::revert(arguments[0], arguments[1], self);
                Vec::new()
            }
            BuiltIn::YulStop => {
                Word::stop(self);
                Vec::new()
            }
            BuiltIn::YulInvalid => {
                Word::invalid(self);
                Vec::new()
            }
            BuiltIn::YulSelfdestruct => {
                Word::selfdestruct(arguments[0], self);
                Vec::new()
            }
            // `log0` through `log4` share one built-in; the topic count is the operand count.
            BuiltIn::YulLog => {
                Word::log(arguments[0], arguments[1], &arguments[2..], self);
                Vec::new()
            }

            BuiltIn::YulAddress => one(Word::address(self)),
            BuiltIn::YulBalance => one(Word::balance(arguments[0], self)),
            BuiltIn::YulSelfbalance => one(Word::selfbalance(self)),
            BuiltIn::YulCaller => one(Word::caller(self)),
            BuiltIn::YulCallvalue => one(Word::callvalue(self)),
            BuiltIn::YulGas => one(Word::gas(self)),
            BuiltIn::YulGasprice => one(Word::gasprice(self)),
            BuiltIn::YulGaslimit => one(Word::gaslimit(self)),
            BuiltIn::YulOrigin => one(Word::origin(self)),
            BuiltIn::YulChainid => one(Word::chainid(self)),
            BuiltIn::YulBasefee => one(Word::basefee(self)),
            BuiltIn::YulBlobbasefee => one(Word::blobbasefee(self)),
            BuiltIn::YulCoinbase => one(Word::coinbase(self)),
            BuiltIn::YulTimestamp => one(Word::timestamp(self)),
            BuiltIn::YulNumber => one(Word::number(self)),
            BuiltIn::YulPrevrandao => one(Word::prevrandao(self)),
            BuiltIn::YulBlockhash => one(Word::blockhash(arguments[0], self)),
            BuiltIn::YulBlobhash => one(Word::blobhash(arguments[0], self)),
            // Opcode 0x44 was renamed `prevrandao` at Paris and `difficulty` retired with it.
            // solx admits builtins at `EvmTarget::LATEST`, so slang rejects this one whatever
            // `--evm-version` says, and the catch-all's message would misreport it as not a Yul
            // builtin.
            BuiltIn::YulDifficulty => unreachable!("`difficulty` was deprecated in Paris"),
            built_in => unreachable!("{built_in:?} is not a Yul builtin"),
        }
    }

    /// A Yul comparison, whose signedness the builtin's name carries rather than its operands.
    fn yul_compare(
        &mut self,
        arguments: &[Word<'context>],
        predicate: YulCmpPredicate,
    ) -> Word<'context> {
        Word::compare(arguments[0], arguments[1], predicate, self)
    }
}
