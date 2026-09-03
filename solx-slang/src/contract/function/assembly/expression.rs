//!
//! Yul expression lowering: literals, path reads, builtin calls, and calls into the block's own Yul
//! functions.
//!

use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::YulExpression;
use slang_solidity_v2::ast::YulFunctionCallExpression;

use solx_mlir::Word;
use solx_mlir::YulCmpPredicate;
use solx_mlir::YulFunction;

use crate::scope::assembly::AssemblyScope;

impl<'function, 'contract, 'source_unit, 'context>
    AssemblyScope<'function, 'contract, 'source_unit, 'context>
{
    /// A Yul expression in value position.
    pub fn expression(&mut self, node: &YulExpression) -> Word<'context> {
        match node {
            YulExpression::YulLiteral(literal) => Word::constant(&literal.integer_value(), self),
            YulExpression::YulPath(path) => self.reference(path).read(self),
            YulExpression::YulFunctionCallExpression(call) => self
                .call(call)
                .into_iter()
                .next()
                .expect("a Yul call in value position yields a word"),
        }
    }

    /// A Yul expression in multi-value position: only a call to a multi-return Yul function yields
    /// more than one word, and only a call yields none.
    pub fn expression_words(&mut self, node: &YulExpression) -> Vec<Word<'context>> {
        match node {
            YulExpression::YulFunctionCallExpression(call) => self.call(call),
            _ => vec![self.expression(node)],
        }
    }

    /// A Yul expression emitted for its effects, discarding whatever it yields.
    pub fn expression_effect(&mut self, node: &YulExpression) {
        self.expression_words(node);
    }

    /// A Yul comparison, whose signedness the builtin's name carries rather than its operands.
    fn compare(
        &mut self,
        arguments: &[Word<'context>],
        predicate: YulCmpPredicate,
    ) -> Word<'context> {
        Word::compare(arguments[0], arguments[1], predicate, self)
    }

    /// The Yul builtin `built_in` applied to `arguments`. Every builtin takes and yields words, so
    /// the operands are wired positionally; the ones that yield nothing return an empty list.
    ///
    /// `pop` discards the operand the caller already evaluated for its effects, and `log0` through
    /// `log4` share one built-in whose topic count is the operand count.
    fn builtin(&mut self, built_in: BuiltIn, arguments: &[Word<'context>]) -> Vec<Word<'context>> {
        match built_in {
            BuiltIn::YulAdd => vec![Word::add(arguments[0], arguments[1], self)],
            BuiltIn::YulSub => vec![Word::sub(arguments[0], arguments[1], self)],
            BuiltIn::YulMul => vec![Word::mul(arguments[0], arguments[1], self)],
            BuiltIn::YulDiv => vec![Word::div(arguments[0], arguments[1], self)],
            BuiltIn::YulSdiv => vec![Word::sdiv(arguments[0], arguments[1], self)],
            BuiltIn::YulMod => vec![Word::r#mod(arguments[0], arguments[1], self)],
            BuiltIn::YulSmod => vec![Word::smod(arguments[0], arguments[1], self)],
            BuiltIn::YulExp => vec![Word::exp(arguments[0], arguments[1], self)],
            BuiltIn::YulAddmod => {
                vec![Word::addmod(arguments[0], arguments[1], arguments[2], self)]
            }
            BuiltIn::YulMulmod => {
                vec![Word::mulmod(arguments[0], arguments[1], arguments[2], self)]
            }
            BuiltIn::YulSignextend => vec![Word::signextend(arguments[0], arguments[1], self)],

            BuiltIn::YulAnd => vec![Word::and(arguments[0], arguments[1], self)],
            BuiltIn::YulOr => vec![Word::or(arguments[0], arguments[1], self)],
            BuiltIn::YulXor => vec![Word::xor(arguments[0], arguments[1], self)],
            BuiltIn::YulNot => vec![Word::not(arguments[0], self)],
            BuiltIn::YulShl => vec![Word::shl(arguments[0], arguments[1], self)],
            BuiltIn::YulShr => vec![Word::shr(arguments[0], arguments[1], self)],
            BuiltIn::YulSar => vec![Word::sar(arguments[0], arguments[1], self)],
            BuiltIn::YulClz => vec![Word::clz(arguments[0], self)],
            BuiltIn::YulByte => vec![Word::byte(arguments[0], arguments[1], self)],

            BuiltIn::YulLt => vec![self.compare(arguments, YulCmpPredicate::UnsignedLt)],
            BuiltIn::YulGt => vec![self.compare(arguments, YulCmpPredicate::UnsignedGt)],
            BuiltIn::YulSlt => vec![self.compare(arguments, YulCmpPredicate::SignedLt)],
            BuiltIn::YulSgt => vec![self.compare(arguments, YulCmpPredicate::SignedGt)],
            BuiltIn::YulEq => vec![self.compare(arguments, YulCmpPredicate::Eq)],
            BuiltIn::YulIszero => {
                let zero = Word::zero(self);
                vec![Word::compare(arguments[0], zero, YulCmpPredicate::Eq, self)]
            }
            BuiltIn::YulPop => Vec::new(),

            BuiltIn::YulMload => vec![Word::mload(arguments[0], self)],
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
            BuiltIn::YulMsize => vec![Word::msize(self)],
            BuiltIn::YulKeccak256 => vec![Word::keccak256(arguments[0], arguments[1], self)],

            BuiltIn::YulSload => vec![Word::sload(arguments[0], self)],
            BuiltIn::YulSstore => {
                Word::sstore(arguments[0], arguments[1], self);
                Vec::new()
            }
            BuiltIn::YulTload => vec![Word::tload(arguments[0], self)],
            BuiltIn::YulTstore => {
                Word::tstore(arguments[0], arguments[1], self);
                Vec::new()
            }

            BuiltIn::YulCalldataload => vec![Word::calldataload(arguments[0], self)],
            BuiltIn::YulCalldatasize => vec![Word::calldatasize(self)],
            BuiltIn::YulCalldatacopy => {
                Word::calldatacopy(arguments[0], arguments[1], arguments[2], self);
                Vec::new()
            }
            BuiltIn::YulReturndatasize => vec![Word::returndatasize(self)],
            BuiltIn::YulReturndatacopy => {
                Word::returndatacopy(arguments[0], arguments[1], arguments[2], self);
                Vec::new()
            }
            BuiltIn::YulCodesize => vec![Word::codesize(self)],
            BuiltIn::YulCodecopy => {
                Word::codecopy(arguments[0], arguments[1], arguments[2], self);
                Vec::new()
            }
            BuiltIn::YulExtcodesize => vec![Word::extcodesize(arguments[0], self)],
            BuiltIn::YulExtcodehash => vec![Word::extcodehash(arguments[0], self)],
            BuiltIn::YulExtcodecopy => {
                Word::extcodecopy(arguments[0], arguments[1], arguments[2], arguments[3], self);
                Vec::new()
            }

            BuiltIn::YulCreate => {
                vec![Word::create(arguments[0], arguments[1], arguments[2], self)]
            }
            BuiltIn::YulCreate2 => vec![Word::create2(
                arguments[0],
                arguments[1],
                arguments[2],
                arguments[3],
                self,
            )],
            BuiltIn::YulCall => vec![Word::call(
                arguments[0],
                arguments[1],
                arguments[2],
                arguments[3],
                arguments[4],
                arguments[5],
                arguments[6],
                self,
            )],
            BuiltIn::YulCallcode => vec![Word::callcode(
                arguments[0],
                arguments[1],
                arguments[2],
                arguments[3],
                arguments[4],
                arguments[5],
                arguments[6],
                self,
            )],
            BuiltIn::YulStaticcall => vec![Word::staticcall(
                arguments[0],
                arguments[1],
                arguments[2],
                arguments[3],
                arguments[4],
                arguments[5],
                self,
            )],
            BuiltIn::YulDelegatecall => vec![Word::delegatecall(
                arguments[0],
                arguments[1],
                arguments[2],
                arguments[3],
                arguments[4],
                arguments[5],
                self,
            )],

            BuiltIn::YulReturn => {
                Word::r#return(arguments[0], arguments[1], self);
                Vec::new()
            }
            BuiltIn::YulRevert => {
                Word::revert(arguments[0], arguments[1], self);
                Vec::new()
            }
            BuiltIn::YulStop => {
                self.current_block().stop(self);
                Vec::new()
            }
            BuiltIn::YulInvalid => {
                self.current_block().invalid(self);
                Vec::new()
            }
            BuiltIn::YulSelfdestruct => {
                Word::selfdestruct(arguments[0], self);
                Vec::new()
            }
            BuiltIn::YulLog => {
                Word::log(arguments[0], arguments[1], &arguments[2..], self);
                Vec::new()
            }

            BuiltIn::YulAddress => vec![Word::address(self)],
            BuiltIn::YulBalance => vec![Word::balance(arguments[0], self)],
            BuiltIn::YulSelfbalance => vec![Word::selfbalance(self)],
            BuiltIn::YulCaller => vec![Word::caller(self)],
            BuiltIn::YulCallvalue => vec![Word::callvalue(self)],
            BuiltIn::YulGas => vec![Word::gas(self)],
            BuiltIn::YulGasprice => vec![Word::gasprice(self)],
            BuiltIn::YulGaslimit => vec![Word::gaslimit(self)],
            BuiltIn::YulOrigin => vec![Word::origin(self)],
            BuiltIn::YulChainid => vec![Word::chainid(self)],
            BuiltIn::YulBasefee => vec![Word::basefee(self)],
            BuiltIn::YulBlobbasefee => vec![Word::blobbasefee(self)],
            BuiltIn::YulCoinbase => vec![Word::coinbase(self)],
            BuiltIn::YulTimestamp => vec![Word::timestamp(self)],
            BuiltIn::YulNumber => vec![Word::number(self)],
            BuiltIn::YulPrevrandao => vec![Word::prevrandao(self)],
            BuiltIn::YulBlockhash => vec![Word::blockhash(arguments[0], self)],
            BuiltIn::YulBlobhash => vec![Word::blobhash(arguments[0], self)],
            // Opcode 0x44 was renamed `prevrandao` at Paris and `difficulty` retired with it.
            // solx admits builtins at `EvmTarget::LATEST`, so slang rejects this one whatever
            // `--evm-version` says.
            BuiltIn::YulDifficulty => unreachable!("`difficulty` was deprecated in Paris"),
            built_in => unreachable!("{built_in:?} is not a Yul builtin"),
        }
    }

    /// A Yul call: a builtin, or a call into a Yul function the enclosing assembly block declares,
    /// whose `yul.func` this naming emits if no earlier one did.
    fn call(&mut self, node: &YulFunctionCallExpression) -> Vec<Word<'context>> {
        let YulExpression::YulPath(callee) = node.operand() else {
            unreachable!("a Yul call names its callee by path");
        };
        let name = callee
            .iter()
            .next()
            .expect("a Yul callee path names one identifier");
        // Yul evaluates an argument list right to left - the order the EVM pushes them in.
        let mut arguments: Vec<_> = node
            .arguments()
            .iter()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|argument| self.expression(&argument))
            .collect();
        arguments.reverse();

        if let Some(built_in) = name.resolve_to_built_in() {
            return self.builtin(built_in, &arguments);
        }
        let Some(Definition::YulFunction(definition)) = name.resolve_to_definition() else {
            unreachable!(
                "a Yul callee is a builtin or a Yul function: {}",
                name.name()
            );
        };
        let signature = self.function_definition(&definition);
        YulFunction::call(&signature, &arguments, self)
    }
}
