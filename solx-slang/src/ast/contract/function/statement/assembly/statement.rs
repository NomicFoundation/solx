//!
//! Yul statement emission.
//!

use melior::ir::Attribute;
use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::attribute::DenseElementsAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::RankedTensorType;
use num_bigint::BigInt;
use slang_solidity_v2::ast::YulExpression;
use slang_solidity_v2::ast::YulStatement;
use slang_solidity_v2::ast::YulSwitchCase;
use solx_mlir::YulValue;
use solx_mlir::ods::yul::*;

use crate::ast::Emit;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::statement::assembly::YulContext;
use crate::ast::contract::function::statement::assembly::block::EmitRegionBody;

// Yul never diverges solx control flow at the source level, but a `break`/
// `continue` terminates its region — that is the `None` continuation, threaded
// like a Sol statement's.
yul_emit!(YulStatement => Option<BlockRef<'context, 'block>>; |statement, context, block| {
    match statement {
        YulStatement::YulVariableAssignmentStatement(assignment) => {
            let expression = assignment.expression();
            let paths: Vec<_> = assignment.variables().iter().collect();
            let (values, current) = if paths.len() == 1 {
                let (value, current) = expression.emit(context, block);
                (vec![value], current)
            } else {
                let YulExpression::YulFunctionCallExpression(call) = &expression else {
                    unreachable!("multi-value yul assignment requires a call right-hand side");
                };
                call.emit(context, block)
            };
            for (path, value) in paths.iter().zip(values) {
                let declaration = path
                    .iter()
                    .next()
                    .expect("empty yul lvalue path")
                    .resolve_to_definition()
                    .expect("yul lvalue resolves to a declaration")
                    .node_id();
                let slot = AstValue::from(context.environment.variable(declaration))
                    .reinterpret(
                        AstType::llvm_ptr(context.state.builder.context),
                        &context.state.builder,
                        &current,
                    )
                    .into_mlir();
                value.store(slot, &context.state.builder, &current);
            }
            Some(current)
        }
        YulStatement::YulVariableDeclarationStatement(declaration) => {
            let variables = declaration.variables();
            let mut current = block;
            let initials: Vec<Option<YulValue<'context, 'block>>> =
                if let Some(value_node) = declaration.value() {
                    let expression = value_node.expression();
                    if variables.len() > 1 {
                        let YulExpression::YulFunctionCallExpression(call) = &expression else {
                            unreachable!("multi-value yul declaration requires a call right-hand side");
                        };
                        let (values, next) = call.emit(context, current);
                        current = next;
                        values.into_iter().map(Some).collect()
                    } else {
                        let (value, next) = expression.emit(context, current);
                        current = next;
                        vec![Some(value)]
                    }
                } else {
                    (0..variables.len()).map(|_| None).collect()
                };
            let builder = &context.state.builder;
            for (identifier, initial) in variables.iter().zip(initials) {
                let slot = YulValue::alloca(builder, &current);
                let word =
                    initial.unwrap_or_else(|| YulValue::constant(&BigInt::from(0u32), builder, &current));
                word.store(slot, builder, &current);
                context.environment.define_variable(identifier.node_id(), slot);
            }
            Some(current)
        }
        YulStatement::YulExpression(expression) => {
            let (_value, current) = expression.emit(context, block);
            Some(current)
        }
        YulStatement::YulIfStatement(if_statement) => {
            let condition = if_statement.condition();
            let (condition_value, block) = condition.emit(context, block);
            // Yul `if` has no `else`, so the else region stays empty —
            // `mlir_region_op!` would append a block to it, so build by hand.
            let then_region = Region::new();
            then_region.append_block(Block::new(&[]));
            let else_region = Region::new();
            let operation = block.append_operation(mlir_op_build!(
                &context.state.builder,
                IfOperation
                    .cond(condition_value)
                    .then_region(then_region)
                    .else_region(else_region)
                    .results(&[])
            ));
            let then_block = operation
                .region(0)
                .expect("yul.if has a then region")
                .first_block()
                .expect("then region has a block");
            if_statement.body().emit_region_body(context, then_block);
            Some(block)
        }
        YulStatement::YulForStatement(for_statement) => {
            context.environment.enter_scope();
            // Initialization runs once in the parent block; a divergent init makes
            // the loop unreachable (solc permits this silently).
            let mut current = block;
            for inner in for_statement.initialization().statements().iter() {
                match inner.emit(context, current) {
                    Some(next) => current = next,
                    None => {
                        context.environment.exit_scope();
                        return None;
                    }
                }
            }

            let (cond_block, body_block, step_block) = mlir_region_op!(
                &context.state.builder,
                &current,
                ForOperation.init_args(&[]).results(&[]);
                cond, body, step
            );

            let cond_region = cond_block
                .parent_region()
                .expect("yul.for cond block has a parent region");
            let saved_region = context.region_pointer;
            context.set_region(&cond_region);
            let (cond_value, cond_end) = for_statement.condition().emit(context, cond_block);
            mlir_op_void!(
                &context.state.builder,
                &cond_end,
                ConditionOperation.condition(cond_value).args(&[])
            );
            context.region_pointer = saved_region;

            for_statement.body().emit_region_body(context, body_block);
            for_statement.iterator().emit_region_body(context, step_block);

            context.environment.exit_scope();
            Some(current)
        }
        YulStatement::YulBlock(yul_block) => {
            context.environment.enter_scope();
            let result = yul_block.emit(context, block);
            context.environment.exit_scope();
            result
        }
        YulStatement::YulFunctionDefinition(_) => {
            // Pre-registered by the enclosing block's hoisting pre-pass.
            Some(block)
        }
        YulStatement::YulLeaveStatement(_) => {
            // `leave` returns from the current Yul function; the inliner stops
            // emitting a body at `leave`, so here it is a no-op.
            Some(block)
        }
        YulStatement::YulBreakStatement(_) => {
            mlir_op_void!(&context.state.builder, &block, BreakOperation);
            None
        }
        YulStatement::YulContinueStatement(_) => {
            mlir_op_void!(&context.state.builder, &block, ContinueOperation);
            None
        }
        YulStatement::YulSwitchStatement(switch_statement) => {
            let (selector, current) = switch_statement.expression().emit(context, block);
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

            // A value-less `switch X default { … }` runs the default body
            // unconditionally, with no `yul.switch`.
            if value_cases.is_empty() {
                return match default_body {
                    Some(default_body) => default_body.emit(context, current),
                    None => Some(current),
                };
            }

            let context_handle = context.state.builder.context;
            let case_attributes: Vec<Attribute<'context>> = value_cases
                .iter()
                .map(|case| YulValue::word_attribute(&case.value().value(), context_handle))
                .collect();
            let cases_type = RankedTensorType::new(
                &[value_cases.len() as u64],
                AstType::signless(context_handle, solx_utils::BIT_LENGTH_FIELD).into_mlir(),
                None,
            )
            .into();
            let cases = DenseElementsAttribute::new(cases_type, &case_attributes)
                .expect("valid i256 switch-case elements");

            let default_region = Region::new();
            default_region.append_block(Block::new(&[]));
            let mut case_regions = Vec::with_capacity(value_cases.len());
            for _ in value_cases.iter() {
                let case_region = Region::new();
                case_region.append_block(Block::new(&[]));
                case_regions.push(case_region);
            }

            let operation = current.append_operation(mlir_op_build!(
                &context.state.builder,
                SwitchOperation
                    .arg(selector)
                    .cases(Attribute::from(cases))
                    .default_region(default_region)
                    .case_regions(case_regions)
                    .results(&[])
            ));
            let default_block = operation
                .region(0)
                .expect("yul.switch has a default region")
                .first_block()
                .expect("default region has a block");
            let case_blocks: Vec<_> = (0..value_cases.len())
                .map(|index| {
                    operation
                        .region(index + 1)
                        .expect("yul.switch has the case region")
                        .first_block()
                        .expect("case region has a block")
                })
                .collect();

            for (case, case_block) in value_cases.iter().zip(case_blocks) {
                case.body().emit_region_body(context, case_block);
            }
            match default_body {
                Some(default_body) => default_body.emit_region_body(context, default_block),
                None => {
                    mlir_op_void!(&context.state.builder, &default_block, YieldOperation.operands(&[]));
                }
            }
            Some(current)
        }
    }
});
