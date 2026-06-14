//!
//! Inline-assembly (Yul) statement lowering.
//!

/// Yul EVM-opcode intrinsic lowering.
pub mod intrinsic;

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::RegionLike;
use melior::ir::Value;
use num_bigint::BigInt;
use slang_solidity_v2::ast::AssemblyStatement;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::YulBlock;
use slang_solidity_v2::ast::YulExpression;
use slang_solidity_v2::ast::YulPath;
use slang_solidity_v2::ast::YulStatement;
use slang_solidity_v2::ast::YulSwitchCase;
use slang_solidity_v2::ast::YulValueCase;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::statement::StatementContext;

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
    /// Emits one Yul statement — an exhaustive `match` over all 11
    /// [`YulStatement`] variants (no `_`). Returns `BlockRef` (NOT `Option`):
    /// Yul never terminates solx control flow.
    pub fn emit_yul_statement(
        &mut self,
        statement: &YulStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        match statement {
            YulStatement::YulVariableAssignmentStatement(assignment) => {
                let expression = assignment.expression();
                let variables = assignment.variables();
                if variables.len() != 1 {
                    return self.emit_yul_multi_assignment(&variables, &expression, block);
                }
                let (value, block) = self.emit_yul_expression(&expression, block)?;
                let path = variables.iter().next().expect("len checked to be 1 above");
                self.emit_yul_store_to_path(&path, value, block);
                Ok(block)
            }
            YulStatement::YulVariableDeclarationStatement(declaration) => {
                let variables = declaration.variables();
                let mut current = block;
                let initials: Vec<Option<Value<'context, 'block>>> =
                    if let Some(value_node) = declaration.value() {
                        let expression = value_node.expression();
                        if variables.len() > 1 {
                            let (values, next) =
                                self.emit_yul_multi_call(&expression, variables.len(), current)?;
                            current = next;
                            values.into_iter().map(Some).collect()
                        } else {
                            let (value, next) = self.emit_yul_expression(&expression, current)?;
                            current = next;
                            vec![Some(value)]
                        }
                    } else {
                        (0..variables.len()).map(|_| None).collect()
                    };
                let builder = &self.state.builder;
                for (identifier, initial) in variables.iter().zip(initials) {
                    let pointer = builder.emit_yul_local_alloca(&current);
                    let stored = initial.unwrap_or_else(|| {
                        builder.emit_yul_constant(&BigInt::from(0u32), &current)
                    });
                    builder.emit_yul_local_store(stored, pointer, &current);
                    self.environment
                        .define_variable(identifier.node_id(), pointer);
                }
                Ok(current)
            }
            YulStatement::YulExpression(expression) => {
                let (_value, block) = self.emit_yul_expression(expression, block)?;
                Ok(block)
            }
            YulStatement::YulIfStatement(if_statement) => {
                let condition = if_statement.condition();
                let (condition_value, block) = self.emit_yul_expression(&condition, block)?;
                let then_block = self.state.builder.emit_yul_if(condition_value, &block);
                self.emit_yul_body_in_region(then_block, &if_statement.body())?;
                Ok(block)
            }
            YulStatement::YulForStatement(for_statement) => {
                self.environment.enter_scope();
                // Initialization runs once in the parent block; a terminator in
                // it makes the loop unreachable (solc permits this silently).
                let mut current = block;
                let mut init_terminated = false;
                for inner in for_statement.initialization().statements().iter() {
                    current = self.emit_yul_statement(&inner, current)?;
                    if Self::is_terminating_yul_statement(&inner) {
                        init_terminated = true;
                        break;
                    }
                }
                if init_terminated {
                    self.environment.exit_scope();
                    return Ok(current);
                }

                let (cond_block, body_block, step_block) =
                    self.state.builder.emit_yul_for(&current);
                let cond_region = cond_block
                    .parent_region()
                    .expect("block belongs to a region");
                let saved_region = self.region_pointer;

                self.set_region(&cond_region);
                let cond_expression = for_statement.condition();
                let (cond_value, cond_end) =
                    self.emit_yul_expression(&cond_expression, cond_block)?;
                self.state.builder.emit_yul_condition(cond_value, &cond_end);
                self.region_pointer = saved_region;

                self.emit_yul_body_in_region(body_block, &for_statement.body())?;
                self.emit_yul_body_in_region(step_block, &for_statement.iterator())?;

                self.environment.exit_scope();
                Ok(current)
            }
            YulStatement::YulBlock(yul_block) => {
                self.environment.enter_scope();
                let current = self.emit_yul_statements_hoisted(yul_block, block)?;
                self.environment.exit_scope();
                Ok(current)
            }
            YulStatement::YulFunctionDefinition(_) => {
                // Pre-registered by the enclosing block's hoisting pre-pass.
                Ok(block)
            }
            YulStatement::YulLeaveStatement(_) => {
                // `leave` returns from the current Yul function. With the inline
                // strategy there is no frame to pop; the inliner stops emitting
                // a body at `leave`, so here it is a no-op.
                Ok(block)
            }
            YulStatement::YulBreakStatement(_) => {
                self.state.builder.emit_yul_break(&block);
                Ok(block)
            }
            YulStatement::YulContinueStatement(_) => {
                self.state.builder.emit_yul_continue(&block);
                Ok(block)
            }
            YulStatement::YulSwitchStatement(switch_statement) => {
                let expression = switch_statement.expression();
                let (selector, current) = self.emit_yul_expression(&expression, block)?;
                let mut value_cases = Vec::new();
                let mut default_body = None;
                for case in switch_statement.cases().iter() {
                    match case {
                        YulSwitchCase::YulValueCase(value_case) => value_cases.push(value_case),
                        YulSwitchCase::YulDefaultCase(default_case) => {
                            default_body = Some(default_case.body());
                        }
                    }
                }
                self.emit_yul_switch_chain(selector, &value_cases, default_body.as_ref(), current)?;
                Ok(current)
            }
        }
    }

    /// Emits a Yul region body (an `if` branch or a `for`/`switch` body): each
    /// statement in order, stopping after the first terminator, then a closing
    /// `yul.yield`. A `break`/`continue` already terminates the block, so the
    /// required `yul.yield` goes into a fresh (unreachable) trailing block —
    /// exactly as solc emits it.
    pub fn emit_yul_region_statements(
        &mut self,
        body: &YulBlock,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let mut current = block;
        let mut terminated = false;
        for inner in body.statements().iter() {
            current = self.emit_yul_statement(&inner, current)?;
            if Self::is_terminating_yul_statement(&inner) {
                terminated = true;
                break;
            }
        }
        if terminated {
            let region = current
                .parent_region()
                .expect("region body block has a parent region");
            let tail = region.append_block(Block::new(&[]));
            self.state.builder.emit_yul_yield(&tail);
        } else {
            self.state.builder.emit_yul_yield(&current);
        }
        Ok(current)
    }

    /// Emits a Yul region body into the region owning `target_block` — an `if`
    /// branch, a `for` body / step, or a `switch` case / default — switching the
    /// current region for the duration and restoring it after.
    fn emit_yul_body_in_region(
        &mut self,
        target_block: BlockRef<'context, 'block>,
        body: &YulBlock,
    ) -> anyhow::Result<()> {
        let region = target_block
            .parent_region()
            .expect("block belongs to a region");
        let saved_region = self.region_pointer;
        self.set_region(&region);
        self.emit_yul_region_statements(body, target_block)?;
        self.region_pointer = saved_region;
        Ok(())
    }

    /// Emits a Yul block's statements with function-definition hoisting:
    /// pre-registers the block's `function` definitions (Yul resolves calls
    /// regardless of textual order), emits each non-definition statement
    /// (stopping after a `break` / `continue` terminator), then drops the
    /// definitions added here so an enclosing scope's stay intact. Does NOT open
    /// a lexical scope — the caller decides: the top-level `assembly` block reuses
    /// the function scope, a nested `{ … }` brackets its own.
    fn emit_yul_statements_hoisted(
        &mut self,
        body: &YulBlock,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let saved_functions: Vec<String> = self.yul_functions.keys().cloned().collect();
        for statement in body.statements().iter() {
            if let YulStatement::YulFunctionDefinition(definition) = &statement {
                self.yul_functions
                    .insert(definition.name().name(), definition.clone());
            }
        }
        let mut current = block;
        for statement in body.statements().iter() {
            if matches!(statement, YulStatement::YulFunctionDefinition(_)) {
                continue;
            }
            current = self.emit_yul_statement(&statement, current)?;
            if Self::is_terminating_yul_statement(&statement) {
                break;
            }
        }
        let added: Vec<String> = self
            .yul_functions
            .keys()
            .filter(|key| !saved_functions.contains(*key))
            .cloned()
            .collect();
        for key in added {
            self.yul_functions.remove(&key);
        }
        Ok(current)
    }

    /// Lowers a Yul `switch` to a single `yul.switch`: one region per value case
    /// keyed by the case word, plus the default region. A value-less `switch X
    /// default { … }` runs the default body unconditionally (no `yul.switch`).
    pub fn emit_yul_switch_chain(
        &mut self,
        selector: Value<'context, 'block>,
        value_cases: &[YulValueCase],
        default_body: Option<&YulBlock>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        if value_cases.is_empty() {
            if let Some(default_body) = default_body {
                let mut current = block;
                for inner in default_body.statements().iter() {
                    current = self.emit_yul_statement(&inner, current)?;
                }
            }
            return Ok(());
        }

        let case_values: Vec<BigInt> = value_cases
            .iter()
            .map(|case| case.value().value())
            .collect();
        let (default_block, case_blocks) =
            self.state
                .builder
                .emit_yul_switch(selector, &case_values, &block);

        for (case, case_block) in value_cases.iter().zip(case_blocks) {
            self.emit_yul_body_in_region(case_block, &case.body())?;
        }
        match default_body {
            Some(default_body) => self.emit_yul_body_in_region(default_block, default_body)?,
            None => self.state.builder.emit_yul_yield(&default_block),
        }
        Ok(())
    }

    /// Emits one Yul expression — an exhaustive 3-arm `match` over
    /// [`YulExpression`] (Literal / Path / Call), producing an `i256` word. Yul
    /// evaluates call arguments right-to-left; a call whose callee resolves to a
    /// `BuiltIn::Yul*` is an EVM intrinsic, otherwise a user-defined Yul function
    /// (inlined).
    pub fn emit_yul_expression(
        &mut self,
        expression: &YulExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        match expression {
            YulExpression::YulLiteral(literal) => {
                let word = literal.value();
                let constant = self.state.builder.emit_yul_constant(&word, &block);
                Ok((constant, block))
            }
            YulExpression::YulPath(path) => self.emit_yul_path(path, block),
            YulExpression::YulFunctionCallExpression(call) => {
                let YulExpression::YulPath(path) = call.operand() else {
                    unimplemented!("unsupported yul callee expression");
                };
                let callee = path.iter().next().expect("empty yul function path");
                let argument_nodes: Vec<_> = call.arguments().iter().collect();
                let mut arguments: Vec<Option<Value<'context, 'block>>> =
                    vec![None; argument_nodes.len()];
                let mut current = block;
                for (index, argument) in argument_nodes.iter().enumerate().rev() {
                    let (value, next) = self.emit_yul_expression(argument, current)?;
                    arguments[index] = Some(value);
                    current = next;
                }
                let arguments: Vec<_> = arguments
                    .into_iter()
                    .map(|value| value.expect("filled in the loop above"))
                    .collect();
                match callee.resolve_to_built_in() {
                    Some(builtin) => self.emit_yul_intrinsic(builtin, &arguments, current),
                    None => self.emit_yul_user_call(&callee.name(), &arguments, current),
                }
            }
        }
    }

    /// Lowers a Yul path read to a 256-bit word: a single-segment path resolves
    /// to a Solidity constant's widened initializer or a local/Yul variable's
    /// loaded value; a two-segment `x.slot` / `x.offset` (keyed by the typed
    /// `BuiltIn::YulSlot` / `BuiltIn::YulOffset` suffix, R8-9 — never the member
    /// name string) resolves to a state variable's slot index / in-slot byte
    /// offset.
    pub fn emit_yul_path(
        &self,
        path: &YulPath,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let identifier = path.iter().next().expect("empty yul path");
        let builder = &self.state.builder;
        let i256 =
            crate::ast::Type::signless(builder.context, solx_utils::BIT_LENGTH_FIELD).into_mlir();

        // A Solidity constant referenced in assembly resolves to a definition,
        // not a Yul/local variable; emit its initializer widened to a word.
        if path.len() == 1
            && let Some(Definition::Constant(constant)) = identifier.resolve_to_definition()
        {
            let initializer = constant.value().expect("constant has no initializer");
            let emitter = ExpressionContext::from(self);
            let BlockAnd { value, block } = initializer.emit(&emitter, block)?;
            let widened = value
                .cast(crate::ast::Type::new(i256), builder, &block)
                .into_mlir();
            return Ok((widened, block));
        }

        // `stateVar.slot` / `stateVar.offset`: the slot index / in-slot byte
        // offset from the storage layout. The suffix is the typed Yul built-in,
        // not the segment text.
        if path.len() == 2 {
            let parts: Vec<_> = path.iter().collect();
            if let Some(Definition::StateVariable(state_variable)) =
                parts[0].resolve_to_definition()
            {
                let slot = self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .expect("unregistered state variable");
                match parts[1].resolve_to_built_in() {
                    Some(BuiltIn::YulSlot) => {
                        let slot_word = BigInt::from_bytes_be(
                            num_bigint::Sign::Plus,
                            &slot.slot.to_be_bytes_vec(),
                        );
                        return Ok((builder.emit_yul_constant(&slot_word, &block), block));
                    }
                    Some(BuiltIn::YulOffset) => {
                        let offset =
                            builder.emit_yul_constant(&BigInt::from(slot.byte_offset), &block);
                        return Ok((offset, block));
                    }
                    _ => {}
                }
            }

            // `localRef.slot` / `localRef.offset` for a `storage` reference local
            // (`T storage x = …`): the local stores the slot index, and a storage
            // reference is slot-aligned, so the in-slot byte offset is 0 (matching
            // solc). Without this, the fall-through below loads the local for both
            // suffixes, so `.offset` wrongly returns the slot.
            if matches!(
                parts[0].resolve_to_definition(),
                Some(Definition::Variable(_) | Definition::Parameter(_))
            ) {
                match parts[1].resolve_to_built_in() {
                    Some(BuiltIn::YulSlot) => {
                        let declaration = parts[0]
                            .resolve_to_definition()
                            .expect("yul path head resolves to a declaration")
                            .node_id();
                        let pointer = self.environment.variable(declaration);
                        let llvm_pointer = self.yul_local_pointer(pointer, &block);
                        let value = builder.emit_yul_local_load(llvm_pointer, &block);
                        return Ok((value, block));
                    }
                    Some(BuiltIn::YulOffset) => {
                        return Ok((
                            builder.emit_yul_constant(&BigInt::from(0u32), &block),
                            block,
                        ));
                    }
                    _ => {}
                }
            }
        }

        let declaration = identifier
            .resolve_to_definition()
            .expect("yul variable reference resolves to a declaration")
            .node_id();
        let pointer = self.environment.variable(declaration);
        let llvm_pointer = self.yul_local_pointer(pointer, &block);
        let value = builder.emit_yul_local_load(llvm_pointer, &block);
        Ok((value, block))
    }

    /// Whether a Yul statement terminates its region (`break` / `continue`);
    /// `yul.return` / `yul.revert` are effects, not terminators.
    pub fn is_terminating_yul_statement(statement: &YulStatement) -> bool {
        matches!(
            statement,
            YulStatement::YulBreakStatement(_) | YulStatement::YulContinueStatement(_)
        )
    }

    /// Emits a call of a user-defined Yul function (single result): the first
    /// return slot, or `0` for a function with no returns. Recursion is rejected
    /// (the inline strategy would loop the compiler).
    pub fn emit_yul_user_call(
        &mut self,
        name: &str,
        arguments: &[Value<'context, 'block>],
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (returns, body_block) = self.emit_yul_inline(name, arguments, block)?;
        let result = returns.into_iter().next().unwrap_or_else(|| {
            self.state
                .builder
                .emit_yul_constant(&BigInt::from(0u32), &body_block)
        });
        Ok((result, body_block))
    }

    /// Emits a call of a user-defined Yul function returning every declared
    /// return slot (for multi-result `let`/assignment). Recursion is rejected.
    pub fn emit_yul_user_call_multi(
        &mut self,
        name: &str,
        arguments: &[Value<'context, 'block>],
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        self.emit_yul_inline(name, arguments, block)
    }

    /// Inlines a user-defined Yul function: binds parameters and zero-initialised
    /// return slots in a fresh scope (each a `llvm.alloca` Yul local), hoists
    /// nested function definitions, lowers the body (stopping at a `leave`), then
    /// reads back the return slots as words.
    ///
    /// `solc`'s own MLIR backend asserts on user-defined Yul functions, so there
    /// is no ground truth to mirror; inlining keeps behavioural parity while
    /// staying within the Yul dialect.
    fn emit_yul_inline(
        &mut self,
        name: &str,
        arguments: &[Value<'context, 'block>],
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let depth = self.yul_inline_depth.entry(name.to_string()).or_insert(0);
        if *depth >= 1 {
            unimplemented!("recursive yul function `{name}` cannot be inlined");
        }
        *depth += 1;
        let definition = self
            .yul_functions
            .get(name)
            .cloned()
            .expect("yul function not registered");

        let parameters: Vec<_> = definition.parameters().iter().collect();
        let returns: Vec<_> = definition
            .returns()
            .map(|names| names.iter().collect::<Vec<_>>())
            .unwrap_or_default();
        assert!(
            arguments.len() == parameters.len(),
            "yul call `{name}` arity mismatch: {} args vs {} params",
            arguments.len(),
            parameters.len()
        );

        let builder = &self.state.builder;
        self.environment.enter_scope();
        for (parameter, argument) in parameters.iter().zip(arguments.iter()) {
            let pointer = builder.emit_yul_local_alloca(&block);
            builder.emit_yul_local_store(*argument, pointer, &block);
            self.environment
                .define_variable(parameter.node_id(), pointer);
        }
        for return_identifier in &returns {
            let pointer = builder.emit_yul_local_alloca(&block);
            let zero = builder.emit_yul_constant(&BigInt::from(0u32), &block);
            builder.emit_yul_local_store(zero, pointer, &block);
            self.environment
                .define_variable(return_identifier.node_id(), pointer);
        }

        // Yul hoists nested functions: register them for the duration of this
        // frame so calls resolve regardless of textual order.
        let mut hoisted: Vec<String> = Vec::new();
        for inner in definition.body().statements().iter() {
            if let YulStatement::YulFunctionDefinition(nested) = &inner {
                let nested_name = nested.name().name();
                if !self.yul_functions.contains_key(&nested_name) {
                    self.yul_functions
                        .insert(nested_name.clone(), nested.clone());
                    hoisted.push(nested_name);
                }
            }
        }
        let mut current = block;
        for inner in definition.body().statements().iter() {
            if matches!(inner, YulStatement::YulFunctionDefinition(_)) {
                continue;
            }
            // `leave` returns from the function: stop emitting the body.
            if matches!(inner, YulStatement::YulLeaveStatement(_)) {
                break;
            }
            current = self.emit_yul_statement(&inner, current)?;
            if Self::is_terminating_yul_statement(&inner) {
                break;
            }
        }
        for nested_name in &hoisted {
            self.yul_functions.remove(nested_name);
        }

        let mut return_values = Vec::with_capacity(returns.len());
        for return_identifier in &returns {
            let pointer = self.environment.variable(return_identifier.node_id());
            let loaded = self.state.builder.emit_yul_local_load(pointer, &current);
            return_values.push(loaded);
        }
        self.environment.exit_scope();
        if let Some(depth) = self.yul_inline_depth.get_mut(name) {
            *depth = depth.saturating_sub(1);
        }
        Ok((return_values, current))
    }

    /// Reinterprets a variable's slot pointer as the `!llvm.ptr` that Yul
    /// `llvm.load`/`llvm.store` operate on. A Yul local's slot is already an
    /// `!llvm.ptr`; a Solidity variable's `!sol.ptr<…, Stack>` crosses the
    /// boundary through a `sol.conv_cast`.
    fn yul_local_pointer(
        &self,
        pointer: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let builder = &self.state.builder;
        crate::ast::Value::from(pointer)
            .reinterpret(crate::ast::Type::llvm_ptr(builder.context), builder, block)
            .into_mlir()
    }

    /// Stores `value` (an `i256` word) into the local/Yul variable named by a
    /// single-segment Yul lvalue path.
    fn emit_yul_store_to_path(
        &self,
        path: &YulPath,
        value: Value<'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) {
        let declaration = path
            .iter()
            .next()
            .expect("empty yul lvalue path")
            .resolve_to_definition()
            .expect("yul lvalue resolves to a declaration")
            .node_id();
        let pointer = self.environment.variable(declaration);
        let llvm_pointer = self.yul_local_pointer(pointer, &block);
        self.state
            .builder
            .emit_yul_local_store(value, llvm_pointer, &block);
    }

    /// Emits a multi-target assignment `x, y := f(…)`: the RHS must be a call to
    /// a user-defined Yul function whose return arity matches the lvalue count.
    fn emit_yul_multi_assignment(
        &mut self,
        variables: &slang_solidity_v2::ast::YulPaths,
        expression: &YulExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let (values, current) = self.emit_yul_multi_call(expression, variables.len(), block)?;
        for (path, value) in variables.iter().zip(values) {
            self.emit_yul_store_to_path(&path, value, current);
        }
        Ok(current)
    }

    /// Evaluates a multi-result Yul call (the RHS of a multi-target `let` or
    /// assignment): the callee must be a user-defined function with `expected`
    /// returns. Emits the arguments left-to-right, then inlines the call.
    fn emit_yul_multi_call(
        &mut self,
        expression: &YulExpression,
        expected: usize,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let YulExpression::YulFunctionCallExpression(call) = expression else {
            unreachable!("multi-value yul binding requires a call RHS");
        };
        let YulExpression::YulPath(path) = call.operand() else {
            unreachable!("multi-value yul call has a non-path callee");
        };
        let callee = path.iter().next().expect("empty yul callee path").name();
        if !self.yul_functions.contains_key(&callee) {
            unimplemented!("multi-value yul binding RHS is not a user-defined function");
        }
        let mut arguments = Vec::new();
        let mut current = block;
        for argument in call.arguments().iter() {
            let (value, next) = self.emit_yul_expression(&argument, current)?;
            arguments.push(value);
            current = next;
        }
        let (values, current) = self.emit_yul_user_call_multi(&callee, &arguments, current)?;
        assert!(
            values.len() == expected,
            "yul binding arity mismatch: {expected} targets vs {} results",
            values.len(),
        );
        Ok((values, current))
    }
}

// An `assembly { … }` block is the top-level Yul block, emitted with
// function-definition hoisting (emit_yul_statements_hoisted) while reusing the
// enclosing function scope (no nested lexical scope).
statement_emit!(AssemblyStatement; |node, context, block| {
    let current_block = context.emit_yul_statements_hoisted(&node.body(), block)?;
    Ok(Some(current_block))
});
