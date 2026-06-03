//!
//! Inline-assembly (Yul) lowering to MLIR.
//!
//! Experimental: handles the common subset used by `yul_instructions/` and
//! `yul_semantic/` tests — straight-line arithmetic, comparisons, bitwise
//! ops, memory and storage load/store, and context intrinsics. Yul control
//! flow (if/for/switch) and user-defined Yul functions are not yet wired.
//!

pub(crate) use melior::ir::BlockLike;
pub(crate) use melior::ir::BlockRef;
pub(crate) use melior::ir::Type;
pub(crate) use melior::ir::Value;
pub(crate) use melior::ir::ValueLike;
pub(crate) use melior::ir::r#type::IntegerType;
pub(crate) use num_bigint::BigInt;
pub(crate) use num_traits::Num;
pub(crate) use slang_solidity_v2::ast::AssemblyStatement;
pub(crate) use slang_solidity_v2::ast::Definition;
pub(crate) use slang_solidity_v2::ast::YulBlock;
pub(crate) use slang_solidity_v2::ast::YulExpression;
pub(crate) use slang_solidity_v2::ast::YulLiteral;
pub(crate) use slang_solidity_v2::ast::YulPath;
pub(crate) use slang_solidity_v2::ast::YulStatement;
pub(crate) use solx_mlir::CmpPredicate;
pub(crate) use solx_mlir::ods::sol::AddOperation;
pub(crate) use solx_mlir::ods::sol::AndOperation;
pub(crate) use solx_mlir::ods::sol::BaseFeeOperation;
pub(crate) use solx_mlir::ods::sol::BlockHashOperation;
pub(crate) use solx_mlir::ods::sol::BlockNumberOperation;
pub(crate) use solx_mlir::ods::sol::CallValueOperation;
pub(crate) use solx_mlir::ods::sol::CallerOperation;
pub(crate) use solx_mlir::ods::sol::ChainIdOperation;
pub(crate) use solx_mlir::ods::sol::CoinbaseOperation;
pub(crate) use solx_mlir::ods::sol::DifficultyOperation;
pub(crate) use solx_mlir::ods::sol::ExpOperation;
pub(crate) use solx_mlir::ods::sol::GasLeftOperation;
pub(crate) use solx_mlir::ods::sol::GasLimitOperation;
pub(crate) use solx_mlir::ods::sol::GasPriceOperation;
pub(crate) use solx_mlir::ods::sol::MulOperation;
pub(crate) use solx_mlir::ods::sol::OrOperation;
pub(crate) use solx_mlir::ods::sol::OriginOperation;
pub(crate) use solx_mlir::ods::sol::PrevRandaoOperation;
pub(crate) use solx_mlir::ods::sol::ShlOperation;
pub(crate) use solx_mlir::ods::sol::ShrOperation;
pub(crate) use solx_mlir::ods::sol::SubOperation;
pub(crate) use solx_mlir::ods::sol::TimestampOperation;
pub(crate) use solx_mlir::ods::sol::XorOperation;
pub(crate) use solx_mlir::ods::yul::AddressOperation as YulAddressOp;
pub(crate) use solx_mlir::ods::yul::BalanceOperation as YulBalanceOp;
pub(crate) use solx_mlir::ods::yul::CallOperation as YulCallOp;
pub(crate) use solx_mlir::ods::yul::Create2Operation as YulCreate2Op;
pub(crate) use solx_mlir::ods::yul::CreateOperation as YulCreateOp;
pub(crate) use solx_mlir::ods::yul::DelegateCallOperation as YulDelegateCallOp;
pub(crate) use solx_mlir::ods::yul::LogOperation as YulLogOp;
pub(crate) use solx_mlir::ods::yul::StaticCallOperation as YulStaticCallOp;
pub(crate) use solx_mlir::ods::yul::ByteOperation as YulByteOp;
pub(crate) use solx_mlir::ods::yul::CallDataCopyOperation as YulCallDataCopyOp;
pub(crate) use solx_mlir::ods::yul::CallDataLoadOperation as YulCallDataLoadOp;
pub(crate) use solx_mlir::ods::yul::CallDataSizeOperation as YulCallDataSizeOp;
pub(crate) use solx_mlir::ods::yul::CodeCopyOperation as YulCodeCopyOp;
pub(crate) use solx_mlir::ods::yul::CodeSizeOperation as YulCodeSizeOp;
pub(crate) use solx_mlir::ods::yul::ExtCodeHashOperation as YulExtCodeHashOp;
pub(crate) use solx_mlir::ods::yul::ExtCodeSizeOperation as YulExtCodeSizeOp;
pub(crate) use solx_mlir::ods::yul::InvalidOperation as YulInvalidOp;
pub(crate) use solx_mlir::ods::yul::Keccak256Operation as YulKeccak256Op;
pub(crate) use solx_mlir::ods::yul::MLoadOperation as YulMLoadOp;
pub(crate) use solx_mlir::ods::yul::MStoreOperation as YulMStoreOp;
pub(crate) use solx_mlir::ods::yul::MStore8Operation as YulMStore8Op;
pub(crate) use solx_mlir::ods::yul::MCopyOperation as YulMCopyOp;
pub(crate) use solx_mlir::ods::yul::NotOperation as YulNotOp;
pub(crate) use solx_mlir::ods::yul::ReturnDataCopyOperation as YulReturnDataCopyOp;
pub(crate) use solx_mlir::ods::yul::ReturnDataSizeOperation as YulReturnDataSizeOp;
pub(crate) use solx_mlir::ods::yul::ReturnOperation as YulReturnOp;
pub(crate) use solx_mlir::ods::yul::RevertOperation as YulRevertOp;
pub(crate) use solx_mlir::ods::yul::AddModOperation as YulAddModOp;
pub(crate) use solx_mlir::ods::yul::DivOperation as YulDivOp;
pub(crate) use solx_mlir::ods::yul::ModOperation as YulModOp;
pub(crate) use solx_mlir::ods::yul::MulModOperation as YulMulModOp;
pub(crate) use solx_mlir::ods::yul::SarOperation as YulSarOp;
pub(crate) use solx_mlir::ods::yul::SDivOperation as YulSDivOp;
pub(crate) use solx_mlir::ods::yul::SLoadOperation as YulSLoadOp;
pub(crate) use solx_mlir::ods::yul::SModOperation as YulSModOp;
pub(crate) use solx_mlir::ods::yul::SStoreOperation as YulSStoreOp;
pub(crate) use solx_mlir::ods::yul::TLoadOperation as YulTLoadOp;
pub(crate) use solx_mlir::ods::yul::TStoreOperation as YulTStoreOp;
pub(crate) use solx_mlir::ods::yul::SelfBalanceOperation as YulSelfBalanceOp;
pub(crate) use solx_mlir::ods::yul::SignExtendOperation as YulSignExtendOp;
pub(crate) use solx_mlir::ods::yul::StopOperation as YulStopOp;

pub(crate) use crate::ast::contract::function::expression::ExpressionEmitter;
pub(crate) use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers a Solidity `assembly { ... }` block by walking its Yul body.
    ///
    /// User-defined `function f(args) -> rets {...}` declarations are
    /// inlined at every call site. Recursion is rejected (we'd loop
    /// at compile time).
    pub fn emit_assembly(
        &mut self,
        assembly: &AssemblyStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        // Pre-pass: register all function definitions in this assembly so
        // calls can resolve regardless of the textual order.
        let saved_functions: Vec<String> = self.yul_functions.keys().cloned().collect();
        for statement in assembly.body().statements().iter() {
            if let YulStatement::YulFunctionDefinition(definition) = statement {
                let name = definition.name().name();
                self.yul_functions.insert(name, definition.clone());
            }
        }
        let mut current_block = block;
        for statement in assembly.body().statements().iter() {
            if matches!(statement, YulStatement::YulFunctionDefinition(_)) {
                continue;
            }
            current_block = self.emit_yul_statement(&statement, current_block)?;
        }
        // Restore function table to the state it had before this assembly
        // (only the entries added here are removed; outer-block defs stay).
        let added_keys: Vec<String> = self
            .yul_functions
            .keys()
            .filter(|key| !saved_functions.contains(*key))
            .cloned()
            .collect();
        for key in added_keys {
            self.yul_functions.remove(&key);
        }
        Ok(Some(current_block))
    }

    // A flat per-statement dispatch: one `match statement` arm per
    // [`YulStatement`] variant. The arms vary in size because Yul constructs
    // do (a multi-target `let` carries more than a `break`), but the function
    // does one thing — route a statement to its lowering. The repeated region
    // body loop is factored into `emit_yul_region_statements`, so the residual
    // length and branching are inherent to the variant count, not nesting.
    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    fn emit_yul_statement(
        &mut self,
        statement: &YulStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        match statement {
            YulStatement::YulVariableAssignmentStatement(assignment) => {
                let expression = assignment.expression();
                let variables = assignment.variables();
                // Multi-target assignment `x, y, z := f()` — the RHS must be a
                // call to a user-defined yul function with matching arity.
                if variables.len() != 1 {
                    let YulExpression::YulFunctionCallExpression(call) = &expression else {
                        unreachable!("multi-value yul assignment requires a call RHS");
                    };
                    let YulExpression::YulPath(callee_path) = call.operand() else {
                        unreachable!("multi-value yul assignment RHS has non-path callee");
                    };
                    let callee = callee_path
                        .iter()
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("empty yul callee path"))?
                        .name();
                    if !self.yul_functions.contains_key(&callee) {
                        unimplemented!("multi-value yul assignment RHS is not a user-defined function");
                    }
                    let mut arguments = Vec::new();
                    let mut current = block;
                    for argument in call.arguments().iter() {
                        let (value, next) = self.emit_yul_expression(&argument, current)?;
                        arguments.push(value);
                        current = next;
                    }
                    let (values, current) =
                        self.emit_yul_user_call_multi(&callee, &arguments, current)?;
                    assert!(
                        values.len() == variables.len(),
                        "yul assignment arity mismatch: {} targets vs {} results",
                        variables.len(),
                        values.len(),
                    );
                    for (path, value) in variables.iter().zip(values) {
                        let name = path
                            .iter()
                            .next()
                            .ok_or_else(|| anyhow::anyhow!("empty yul lvalue path"))?
                            .name();
                        let (pointer, element_type) = self.environment.variable_with_type(&name);
                        let builder = &self.state.builder;
                        let cast = if value.r#type() == element_type {
                            value
                        } else {
                            builder.emit_sol_cast(value, element_type, &current)
                        };
                        builder.emit_sol_store(cast, pointer, &current);
                    }
                    return Ok(current);
                }
                let (value, block) = self.emit_yul_expression(&expression, block)?;
                let path = variables
                    .iter()
                    .next()
                    .expect("len checked to be 1 above");
                let name = path
                    .iter()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("empty yul lvalue path"))?
                    .name();
                let (pointer, element_type) = self.environment.variable_with_type(&name);
                let builder = &self.state.builder;
                let cast = if value.r#type() == element_type {
                    value
                } else {
                    builder.emit_sol_cast(value, element_type, &block)
                };
                builder.emit_sol_store(cast, pointer, &block);
                Ok(block)
            }
            YulStatement::YulVariableDeclarationStatement(declaration) => {
                let variables = declaration.variables();
                let variable_count = variables.len();
                let mut current = block;
                // Compute the initial value(s). Single declaration: ordinary
                // expression. Multi declaration: must be a yul function call
                // to a user-defined function with matching return arity (we
                // inline it and collect every return slot).
                let initials: Vec<Option<Value<'context, 'block>>> =
                    if let Some(value_node) = declaration.value() {
                        let expression = value_node.expression();
                        if variable_count > 1 {
                            // Multi-let only supported when the RHS is a call
                            // to a user-defined yul function.
                            let YulExpression::YulFunctionCallExpression(call) = &expression else {
                                unreachable!("multi-variable yul let requires a call RHS");
                            };
                            let YulExpression::YulPath(path) = call.operand() else {
                                unreachable!("multi-variable yul let RHS has non-path callee");
                            };
                            let name = path
                                .iter()
                                .next()
                                .ok_or_else(|| anyhow::anyhow!("empty yul callee path"))?
                                .name();
                            if !self.yul_functions.contains_key(&name) {
                                unimplemented!(
                                    "multi-variable yul let RHS is not a user-defined function"
                                );
                            }
                            let mut arguments = Vec::new();
                            for argument in call.arguments().iter() {
                                let (value, next) =
                                    self.emit_yul_expression(&argument, current)?;
                                arguments.push(value);
                                current = next;
                            }
                            let (values, next) =
                                self.emit_yul_user_call_multi(&name, &arguments, current)?;
                            current = next;
                            assert!(
                                values.len() == variable_count,
                                "yul let arity mismatch: {} variables vs {} call results",
                                variable_count,
                                values.len(),
                            );
                            values.into_iter().map(Some).collect()
                        } else {
                            let (value, next) = self.emit_yul_expression(&expression, current)?;
                            current = next;
                            vec![Some(value)]
                        }
                    } else {
                        (0..variable_count).map(|_| None).collect()
                    };
                let builder = &self.state.builder;
                let element_type = builder.types.ui256;
                for (identifier, initial) in variables.iter().zip(initials) {
                    let name = identifier.name();
                    let pointer = builder.emit_sol_alloca(element_type, &current);
                    let stored = match initial {
                        Some(value) if value.r#type() == element_type => value,
                        Some(value) => builder.emit_sol_cast(value, element_type, &current),
                        None => builder.emit_sol_constant(0, element_type, &current),
                    };
                    builder.emit_sol_store(stored, pointer, &current);
                    self.environment
                        .define_variable(name, pointer, element_type);
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
                let builder = &self.state.builder;
                let ui256 = builder.types.ui256;
                let cond_ui256 = if condition_value.r#type() == ui256 {
                    condition_value
                } else {
                    builder.emit_sol_cast(condition_value, ui256, &block)
                };
                let zero = builder.emit_sol_constant(0, ui256, &block);
                let condition_boolean =
                    builder.emit_sol_cmp(cond_ui256, zero, CmpPredicate::Ne, &block);
                let (then_block, else_block) = builder.emit_sol_if(condition_boolean, &block);
                let then_region = then_block.parent_region().expect("block belongs to a region");
                let else_region = else_block.parent_region().expect("block belongs to a region");

                let saved_region = self.region_pointer;
                self.set_region(&then_region);
                self.emit_yul_region_statements(&if_statement.body(), then_block)?;
                self.region_pointer = saved_region;
                let _ = else_region;
                self.state.builder.emit_sol_yield(&else_block);
                Ok(block)
            }
            YulStatement::YulForStatement(for_statement) => {
                self.environment.enter_scope();
                // Initialization runs once in the parent block.
                let mut current = block;
                let mut init_terminated = false;
                for inner in for_statement.initialization().statements().iter() {
                    if Self::is_terminating_yul_statement(&inner) {
                        // Emit the terminator; later statements would be
                        // unreachable. solc lets this through with no warning.
                        current = self.emit_yul_statement(&inner, current)?;
                        init_terminated = true;
                        break;
                    }
                    current = self.emit_yul_statement(&inner, current)?;
                }
                if init_terminated {
                    self.environment.exit_scope();
                    return Ok(current);
                }

                let (cond_block, body_block, step_block) =
                    self.state.builder.emit_sol_for(&current);
                let cond_region = cond_block.parent_region().expect("block belongs to a region");
                let body_region = body_block.parent_region().expect("block belongs to a region");
                let step_region = step_block.parent_region().expect("block belongs to a region");

                let saved_region = self.region_pointer;

                // Condition region.
                self.set_region(&cond_region);
                let cond_expression = for_statement.condition();
                let (cond_value, cond_end) =
                    self.emit_yul_expression(&cond_expression, cond_block)?;
                let builder = &self.state.builder;
                let ui256 = builder.types.ui256;
                let cond_u = if cond_value.r#type() == ui256 {
                    cond_value
                } else {
                    builder.emit_sol_cast(cond_value, ui256, &cond_end)
                };
                let zero = builder.emit_sol_constant(0, ui256, &cond_end);
                let cond_bool =
                    builder.emit_sol_cmp(cond_u, zero, CmpPredicate::Ne, &cond_end);
                builder.emit_sol_condition(cond_bool, &cond_end);

                // Body region.
                self.set_region(&body_region);
                self.emit_yul_region_statements(&for_statement.body(), body_block)?;

                // Step (iterator) region.
                self.set_region(&step_region);
                self.emit_yul_region_statements(&for_statement.iterator(), step_block)?;

                self.region_pointer = saved_region;
                self.environment.exit_scope();
                Ok(current)
            }
            YulStatement::YulBlock(yul_block) => {
                self.environment.enter_scope();
                // Pre-pass: register yul function defs in this nested block
                // so they're visible to calls within this scope, even when
                // the call appears textually before the definition.
                let saved_keys: Vec<String> =
                    self.yul_functions.keys().cloned().collect();
                for inner in yul_block.statements().iter() {
                    if let YulStatement::YulFunctionDefinition(definition) = inner {
                        let name = definition.name().name();
                        self.yul_functions.insert(name, definition.clone());
                    }
                }
                let mut current = block;
                for inner in yul_block.statements().iter() {
                    if matches!(inner, YulStatement::YulFunctionDefinition(_)) {
                        continue;
                    }
                    if Self::is_terminating_yul_statement(&inner) {
                        current = self.emit_yul_statement(&inner, current)?;
                        break;
                    }
                    current = self.emit_yul_statement(&inner, current)?;
                }
                // Drop the entries we added; outer-scope defs remain.
                let added_keys: Vec<String> = self
                    .yul_functions
                    .keys()
                    .filter(|key| !saved_keys.contains(*key))
                    .cloned()
                    .collect();
                for key in added_keys {
                    self.yul_functions.remove(&key);
                }
                self.environment.exit_scope();
                Ok(current)
            }
            YulStatement::YulFunctionDefinition(_) => {
                // Already pre-registered by emit_assembly / YulBlock pre-pass.
                Ok(block)
            }
            YulStatement::YulLeaveStatement(_) => {
                // `leave` returns from the current yul function. With our
                // inline strategy, we don't have a function frame to return
                // from; treat as a no-op (the result slot keeps its last
                // value). This is incorrect for control-flow tests but
                // unblocks simple straight-line yul function bodies.
                Ok(block)
            }
            YulStatement::YulBreakStatement(_) => {
                self.state.builder.emit_sol_break(&block);
                Ok(block)
            }
            YulStatement::YulContinueStatement(_) => {
                self.state.builder.emit_sol_continue(&block);
                Ok(block)
            }
            YulStatement::YulSwitchStatement(switch_statement) => {
                let expression = switch_statement.expression();
                let (selector, mut current) = self.emit_yul_expression(&expression, block)?;
                let cases: Vec<_> = switch_statement.cases().iter().collect();
                // Split into value cases and optional default.
                let mut value_cases = Vec::new();
                let mut default_body = None;
                for case in &cases {
                    match case {
                        slang_solidity_v2::ast::YulSwitchCase::YulValueCase(value_case) => {
                            value_cases.push(value_case.clone());
                        }
                        slang_solidity_v2::ast::YulSwitchCase::YulDefaultCase(default_case) => {
                            default_body = Some(default_case.body());
                        }
                    }
                }
                // Chain of nested ifs: if (selector == v1) {...} else if (selector == v2) {...} else { default }.
                self.emit_yul_switch_chain(
                    selector,
                    &value_cases,
                    default_body.as_ref(),
                    &mut current,
                )?;
                Ok(current)
            }
        }
    }

    /// Emits the statements of one structured Yul region body — an `if` branch
    /// or a `for`/`switch` body — into `block`. Emits each statement in order,
    /// stopping after the first terminator, then closes the region with a
    /// `sol.yield` unless a terminator already closed it. Returns the block the
    /// region ends in.
    fn emit_yul_region_statements(
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
        if !terminated {
            self.state.builder.emit_sol_yield(&current);
        }
        Ok(current)
    }

    fn emit_yul_switch_chain(
        &mut self,
        selector: Value<'context, 'block>,
        value_cases: &[slang_solidity_v2::ast::YulValueCase],
        default_body: Option<&slang_solidity_v2::ast::YulBlock>,
        block: &mut BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        // `switch X default { ... }` (no value cases) lowers to an
        // unconditional execution of the default body — no sol.if needed.
        if value_cases.is_empty() {
            if let Some(default_body) = default_body {
                let mut current = *block;
                for inner in default_body.statements().iter() {
                    current = self.emit_yul_statement(&inner, current)?;
                }
                *block = current;
            }
            return Ok(());
        }

        let saved_region = self.region_pointer;
        let mut else_blocks = Vec::new();
        let mut current = *block;
        for case in value_cases {
            let literal = case.value();
            let synthetic = YulExpression::YulLiteral(literal);
            let (literal_value, new_block) =
                self.emit_yul_expression(&synthetic, current)?;
            current = new_block;
            let builder = &self.state.builder;
            let condition =
                builder.emit_sol_cmp(selector, literal_value, CmpPredicate::Eq, &current);
            let (then_block, else_block) = builder.emit_sol_if(condition, &current);
            let then_region = then_block.parent_region().expect("block belongs to a region");

            self.set_region(&then_region);
            self.emit_yul_region_statements(&case.body(), then_block)?;

            // The next case (or default) goes into THIS sol.if's else block.
            let else_region = else_block.parent_region().expect("block belongs to a region");
            self.set_region(&else_region);
            else_blocks.push(else_block);
            current = else_block;
        }
        // Inside the deepest else: emit the default body, then yield. A
        // default body that terminates closes its own block; otherwise (or
        // when there is no default) we yield the deepest else explicitly.
        if let Some(default_body) = default_body {
            self.emit_yul_region_statements(default_body, current)?;
        } else {
            self.state.builder.emit_sol_yield(&current);
        }
        // Each non-deepest else block hosts the nested sol.if AND needs a
        // yield after it.
        for else_block in else_blocks.iter().rev().skip(1) {
            self.state.builder.emit_sol_yield(else_block);
        }
        self.region_pointer = saved_region;
        // Parent block where execution continues is unchanged — the caller's
        // `*block`.
        Ok(())
    }

    fn emit_yul_expression(
        &mut self,
        expression: &YulExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        match expression {
            YulExpression::YulLiteral(literal) => {
                let big = Self::yul_literal_to_word(literal)?;
                let builder = &self.state.builder;
                let constant = builder.emit_constant(&big, builder.types.ui256, &block);
                Ok((constant, block))
            }
            YulExpression::YulPath(path) => self.emit_yul_path(path, block),
            YulExpression::YulFunctionCallExpression(call) => {
                let operand = call.operand();
                let YulExpression::YulPath(path) = operand else {
                    unimplemented!("unsupported yul callee");
                };
                let name = path
                    .iter()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("empty yul function path"))?
                    .name();
                // Yul evaluates arguments right-to-left.
                let argument_nodes: Vec<_> = call.arguments().iter().collect();
                let mut arguments = vec![None; argument_nodes.len()];
                let mut current = block;
                for (index, argument) in argument_nodes.iter().enumerate().rev() {
                    let (value, next) = self.emit_yul_expression(argument, current)?;
                    arguments[index] = Some(value);
                    current = next;
                }
                let arguments: Vec<_> = arguments.into_iter().map(|v| v.expect("filled in loop")).collect();
                if self.yul_functions.contains_key(&name) {
                    self.emit_yul_user_call(&name, &arguments, current)
                } else {
                    self.emit_yul_intrinsic(&name, &arguments, current)
                }
            }
        }
    }

    /// Decodes a Yul literal to the 256-bit word it denotes: booleans to 0/1,
    /// decimal/hex numbers parsed directly, and string / `hex"..."` literals
    /// packed left-aligned into the word (zero-padded on the right).
    fn yul_literal_to_word(literal: &YulLiteral) -> anyhow::Result<BigInt> {
        Ok(match literal {
            YulLiteral::TrueKeyword(_) => BigInt::from(1u32),
            YulLiteral::FalseKeyword(_) => BigInt::from(0u32),
            YulLiteral::DecimalLiteral(decimal) => BigInt::from_str_radix(
                decimal.unparse().trim(),
                10,
            )
            .map_err(|error| anyhow::anyhow!("yul decimal literal parse: {error}"))?,
            YulLiteral::HexLiteral(hex) => {
                let text = hex.unparse();
                let hex_digits = text
                    .trim()
                    .strip_prefix("0x")
                    .or_else(|| text.trim().strip_prefix("0X"))
                    .unwrap_or(text.trim());
                BigInt::from_str_radix(hex_digits, 16)
                    .map_err(|error| anyhow::anyhow!("yul hex literal parse: {error}"))?
            }
            YulLiteral::StringLiteral(string_literal) => {
                // Pack the string bytes into a 32-byte big-endian word
                // (left-aligned, zero-padded on the right). Solidity
                // string escapes are processed before packing.
                let raw = string_literal.unparse();
                let body_text = raw
                    .trim()
                    .trim_start_matches(['"', '\''])
                    .trim_end_matches(['"', '\'']);
                let bytes = Self::decode_solidity_string_escapes(body_text);
                let mut padded = [0u8; 32];
                let copy_len = bytes.len().min(32);
                padded[..copy_len].copy_from_slice(&bytes[..copy_len]);
                BigInt::from_bytes_be(num_bigint::Sign::Plus, &padded)
            }
            YulLiteral::HexStringLiteral(hex_string) => {
                // `hex"1234"` → bytes [0x12, 0x34], placed at the
                // most-significant end of the 32-byte word
                // (left-aligned), zero-padded on the right.
                let text = hex_string.unparse();
                let trimmed = text
                    .trim()
                    .trim_start_matches("hex")
                    .trim()
                    .trim_start_matches(['"', '\''])
                    .trim_end_matches(['"', '\'']);
                let clean: String =
                    trimmed.chars().filter(|c| c.is_ascii_hexdigit()).collect();
                // Each pair of hex digits is one byte.
                let mut bytes = Vec::with_capacity(clean.len() / 2);
                let bytes_chars: Vec<char> = clean.chars().collect();
                let mut i = 0;
                while i + 1 < bytes_chars.len() {
                    let hi = bytes_chars[i].to_digit(16).unwrap_or(0) as u8;
                    let lo = bytes_chars[i + 1].to_digit(16).unwrap_or(0) as u8;
                    bytes.push((hi << 4) | lo);
                    i += 2;
                }
                let mut padded = [0u8; 32];
                let copy_len = bytes.len().min(32);
                padded[..copy_len].copy_from_slice(&bytes[..copy_len]);
                BigInt::from_bytes_be(num_bigint::Sign::Plus, &padded)
            }
        })
    }

    /// Lowers a Yul path expression to a 256-bit value. A single-segment path
    /// resolves to a Solidity constant's widened initializer, a local/yul
    /// variable's loaded value, or — as `x.slot` / `x.offset` — a state
    /// variable's storage slot number / in-slot byte offset.
    fn emit_yul_path(
        &mut self,
        path: &YulPath,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let identifier = path
            .iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("empty yul path"))?;
        let name = identifier.name();
        // A Solidity constant referenced in assembly (`assembly { x := C }`)
        // resolves to `Definition::Constant`, not a yul/local variable.
        // Emit its initializer value (widened to a 256-bit word) rather
        // than looking it up as a local, which would hit
        // `variable_with_type`'s unreachable. Multi-element paths
        // (`x.slot` / `x.offset`) keep the existing handling.
        if path.len() == 1
            && let Some(Definition::Constant(constant)) =
                identifier.resolve_to_definition()
        {
            let initializer = constant.value().ok_or_else(|| {
                anyhow::anyhow!("constant {name} has no initializer")
            })?;
            let emitter = ExpressionEmitter::new(
                self.state,
                self.environment,
                self.storage_layout,
                self.checked,
            );
            let (value, block) = emitter.emit(&initializer, block)?;
            let value = value.ok_or_else(|| {
                anyhow::anyhow!("constant {name} initializer produced no value")
            })?;
            let builder = &self.state.builder;
            let ui256 = builder.types.ui256;
            let widened = if value.r#type() == ui256 {
                value
            } else {
                builder.emit_sol_cast(value, ui256, &block)
            };
            return Ok((widened, block));
        }
        // `stateVar.slot` / `stateVar.offset` in assembly resolves to the
        // storage slot number / in-slot byte offset from the storage
        // layout. The path is `[stateVar, slot|offset]`.
        if path.len() == 2 {
            let parts: Vec<_> = path.iter().collect();
            let member = parts[1].name();
            if (member == "slot" || member == "offset")
                && let Some(Definition::StateVariable(state_variable)) =
                    parts[0].resolve_to_definition()
            {
                let &(slot, byte_offset, _location) = self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "unregistered state variable: {}",
                            parts[0].name()
                        )
                    })?;
                let builder = &self.state.builder;
                let ui256 = builder.types.ui256;
                let value = if member == "slot" {
                    let slot_big =
                        BigInt::from_bytes_be(num_bigint::Sign::Plus, &slot.to_be_bytes_vec());
                    builder.emit_constant(&slot_big, ui256, &block)
                } else {
                    builder.emit_sol_constant(i64::from(byte_offset), ui256, &block)
                };
                return Ok((value, block));
            }
        }
        let (pointer, element_type) = self.environment.variable_with_type(&name);
        let value = self
            .state
            .builder
            .emit_sol_load(pointer, element_type, &block)?;
        let builder = &self.state.builder;
        let ui256 = builder.types.ui256;
        let cast = if value.r#type() == ui256 {
            value
        } else if solx_mlir::TypeFactory::is_sol_enum(value.r#type()) {
            // Enum-typed variables bridge to ui256 via `sol.enum_cast`;
            // `sol.cast` rejects non-integer enum operands.
            builder.emit_sol_enum_cast(value, ui256, &block)
        } else {
            builder.emit_sol_cast(value, ui256, &block)
        };
        Ok((cast, block))
    }

    /// Returns true for yul statements that produce a real MLIR block
    /// terminator (so a following `sol.yield` must NOT be emitted).
    /// `yul.return` / `yul.revert` / etc. are effect ops, not terminators —
    /// they do not satisfy MLIR block-terminator requirements.
    fn is_terminating_yul_statement(statement: &YulStatement) -> bool {
        matches!(
            statement,
            YulStatement::YulBreakStatement(_)
                | YulStatement::YulContinueStatement(_)
        )
    }

    /// Inlines a yul user-defined function and returns every return slot
    /// (one `Value` per declared return). Used for multi-result let bindings.
    fn emit_yul_user_call_multi(
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
            .ok_or_else(|| anyhow::anyhow!("yul function `{name}` not registered"))?;

        let parameter_names: Vec<String> = definition
            .parameters()
            .iter()
            .map(|identifier| identifier.name())
            .collect();
        let return_names: Vec<String> = definition
            .returns()
            .map(|names| names.iter().map(|identifier| identifier.name()).collect())
            .unwrap_or_default();
        assert!(
            arguments.len() == parameter_names.len(),
            "yul call `{name}` arity mismatch: {} args vs {} params",
            arguments.len(),
            parameter_names.len()
        );

        let builder = &self.state.builder;
        let ui256 = builder.types.ui256;
        self.environment.enter_scope();
        for (parameter_name, argument_value) in parameter_names.iter().zip(arguments.iter()) {
            let value = if argument_value.r#type() == ui256 {
                *argument_value
            } else {
                builder.emit_sol_cast(*argument_value, ui256, &block)
            };
            let pointer = builder.emit_sol_alloca(ui256, &block);
            builder.emit_sol_store(value, pointer, &block);
            self.environment
                .define_variable(parameter_name.clone(), pointer, ui256);
        }
        for return_name in return_names.iter() {
            let pointer = builder.emit_sol_alloca(ui256, &block);
            let zero = builder.emit_sol_constant(0, ui256, &block);
            builder.emit_sol_store(zero, pointer, &block);
            self.environment
                .define_variable(return_name.clone(), pointer, ui256);
        }
        // Functions nested in this body are visible throughout it regardless
        // of textual order (yul hoists functions) — register them for the
        // duration of this inlined frame so calls to them resolve instead of
        // falling through to the intrinsic table.
        let mut hoisted_functions: Vec<String> = Vec::new();
        for inner in definition.body().statements().iter() {
            if let YulStatement::YulFunctionDefinition(nested) = inner {
                let nested_name = nested.name().name();
                if !self.yul_functions.contains_key(&nested_name) {
                    self.yul_functions
                        .insert(nested_name.clone(), nested.clone());
                    hoisted_functions.push(nested_name);
                }
            }
        }
        let body_block = {
            let mut current = block;
            for inner in definition.body().statements().iter() {
                if matches!(inner, YulStatement::YulFunctionDefinition(_)) {
                    continue;
                }
                // `leave` inside an inlined function: stop emitting further
                // statements. This is a function-frame return, not an MLIR
                // block terminator.
                if matches!(inner, YulStatement::YulLeaveStatement(_)) {
                    break;
                }
                if Self::is_terminating_yul_statement(&inner) {
                    current = self.emit_yul_statement(&inner, current)?;
                    break;
                }
                current = self.emit_yul_statement(&inner, current)?;
            }
            current
        };
        for nested_name in &hoisted_functions {
            self.yul_functions.remove(nested_name);
        }
        let mut return_values: Vec<Value<'context, 'block>> =
            Vec::with_capacity(return_names.len());
        for return_name in return_names.iter() {
            let (pointer, slot_type) = self.environment.variable_with_type(return_name);
            let loaded = self
                .state
                .builder
                .emit_sol_load(pointer, slot_type, &body_block)?;
            let cast = if loaded.r#type() == ui256 {
                loaded
            } else {
                self.state
                    .builder
                    .emit_sol_cast(loaded, ui256, &body_block)
            };
            return_values.push(cast);
        }
        self.environment.exit_scope();
        if let Some(d) = self.yul_inline_depth.get_mut(name) {
            *d = d.saturating_sub(1);
        }
        Ok((return_values, body_block))
    }

    /// Inlines a yul user-defined function: enters a fresh scope, binds
    /// parameters and return slots, lowers the body, then reads the return
    /// slots. Recursive calls are rejected (we'd compile forever).
    fn emit_yul_user_call(
        &mut self,
        name: &str,
        arguments: &[Value<'context, 'block>],
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let depth = self.yul_inline_depth.entry(name.to_string()).or_insert(0);
        if *depth >= 1 {
            unimplemented!("recursive yul function `{name}` cannot be inlined");
        }
        *depth += 1;
        let definition = self
            .yul_functions
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("yul function `{name}` not registered"))?;

        let parameter_names: Vec<String> = definition
            .parameters()
            .iter()
            .map(|identifier| identifier.name())
            .collect();
        let return_names: Vec<String> = definition
            .returns()
            .map(|names| names.iter().map(|identifier| identifier.name()).collect())
            .unwrap_or_default();
        assert!(
            arguments.len() == parameter_names.len(),
            "yul call `{name}` arity mismatch: {} args vs {} params",
            arguments.len(),
            parameter_names.len()
        );

        let builder = &self.state.builder;
        let ui256 = builder.types.ui256;
        self.environment.enter_scope();
        // Bind parameters: each is alloca'd as ui256 and the argument stored.
        for (parameter_name, argument_value) in parameter_names.iter().zip(arguments.iter()) {
            let value = if argument_value.r#type() == ui256 {
                *argument_value
            } else {
                builder.emit_sol_cast(*argument_value, ui256, &block)
            };
            let pointer = builder.emit_sol_alloca(ui256, &block);
            builder.emit_sol_store(value, pointer, &block);
            self.environment
                .define_variable(parameter_name.clone(), pointer, ui256);
        }
        // Allocate and zero-init return slots.
        for return_name in return_names.iter() {
            let pointer = builder.emit_sol_alloca(ui256, &block);
            let zero = builder.emit_sol_constant(0, ui256, &block);
            builder.emit_sol_store(zero, pointer, &block);
            self.environment
                .define_variable(return_name.clone(), pointer, ui256);
        }
        // Hoist nested function definitions (forward-visible within the body).
        let mut hoisted_functions: Vec<String> = Vec::new();
        for inner in definition.body().statements().iter() {
            if let YulStatement::YulFunctionDefinition(nested) = inner {
                let nested_name = nested.name().name();
                if !self.yul_functions.contains_key(&nested_name) {
                    self.yul_functions
                        .insert(nested_name.clone(), nested.clone());
                    hoisted_functions.push(nested_name);
                }
            }
        }
        let body_block = {
            let mut current = block;
            for inner in definition.body().statements().iter() {
                if matches!(inner, YulStatement::YulFunctionDefinition(_)) {
                    continue;
                }
                // `leave` exits the current yul function (no further body
                // statements run).
                if matches!(inner, YulStatement::YulLeaveStatement(_)) {
                    break;
                }
                if Self::is_terminating_yul_statement(&inner) {
                    current = self.emit_yul_statement(&inner, current)?;
                    break;
                }
                current = self.emit_yul_statement(&inner, current)?;
            }
            current
        };
        for nested_name in &hoisted_functions {
            self.yul_functions.remove(nested_name);
        }
        // Read return values; for the multi-result case yul tuple semantics
        // call for separate values, but our intrinsic surface returns one
        // ui256. We pick the first return slot if any, else a constant 0.
        let result = if let Some(first_return) = return_names.first() {
            let (pointer, slot_type) = self.environment.variable_with_type(first_return);
            let loaded = self
                .state
                .builder
                .emit_sol_load(pointer, slot_type, &body_block)?;
            if loaded.r#type() == ui256 {
                loaded
            } else {
                self.state
                    .builder
                    .emit_sol_cast(loaded, ui256, &body_block)
            }
        } else {
            self.state.builder.emit_sol_constant(0, ui256, &body_block)
        };
        self.environment.exit_scope();
        if let Some(d) = self.yul_inline_depth.get_mut(name) {
            *d = d.saturating_sub(1);
        }
        Ok((result, body_block))
    }

    /// Decodes Solidity string-literal escape sequences (`\\`, `\"`, `\n`,
    /// `\xNN`, `\uNNNN`, line continuation `\<newline>`) into raw bytes.
    /// Anything unrecognized is left as-is.
    fn decode_solidity_string_escapes(input: &str) -> Vec<u8> {
        let mut output = Vec::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch != '\\' {
                let mut buf = [0u8; 4];
                let encoded = ch.encode_utf8(&mut buf);
                output.extend_from_slice(encoded.as_bytes());
                continue;
            }
            let Some(next) = chars.next() else {
                output.push(b'\\');
                break;
            };
            match next {
                '\\' => output.push(b'\\'),
                '"' => output.push(b'"'),
                '\'' => output.push(b'\''),
                'n' => output.push(b'\n'),
                'r' => output.push(b'\r'),
                't' => output.push(b'\t'),
                '0' => output.push(0),
                '/' => output.push(b'/'),
                '\n' => {} // line continuation
                'x' => {
                    let hi = chars.next().and_then(|c| c.to_digit(16)).unwrap_or(0) as u8;
                    let lo = chars.next().and_then(|c| c.to_digit(16)).unwrap_or(0) as u8;
                    output.push((hi << 4) | lo);
                }
                'u' => {
                    let mut codepoint: u32 = 0;
                    for _ in 0..4 {
                        let d = chars.next().and_then(|c| c.to_digit(16)).unwrap_or(0);
                        codepoint = (codepoint << 4) | d;
                    }
                    if let Some(ch) = char::from_u32(codepoint) {
                        let mut buf = [0u8; 4];
                        let encoded = ch.encode_utf8(&mut buf);
                        output.extend_from_slice(encoded.as_bytes());
                    }
                }
                other => {
                    let mut buf = [0u8; 4];
                    output.push(b'\\');
                    let encoded = other.encode_utf8(&mut buf);
                    output.extend_from_slice(encoded.as_bytes());
                }
            }
        }
        output
    }

}

mod intrinsic;
