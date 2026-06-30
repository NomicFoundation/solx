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

use crate::ast::BlockAnd;
use crate::ast::EmitYul;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::statement::assembly::YulContext;
use crate::ast::contract::function::statement::assembly::block::EmitRegionBody;

yul_emit!(YulStatement => Option<BlockRef<'context, 'block>>; |statement, context, block| {
    match statement {
        YulStatement::YulVariableAssignmentStatement(assignment) => {
            let expression = assignment.expression();
            let paths: Vec<_> = assignment.variables().iter().collect();
            let (values, current) = if paths.len() == 1 {
                let BlockAnd { value, block: current } = expression.emit(context, block);
                (vec![value], current)
            } else {
                let YulExpression::YulFunctionCallExpression(call) = &expression else {
                    unreachable!("multi-value yul assignment requires a call right-hand side");
                };
                let BlockAnd { value, block } = call.emit(context, block);
                (value, block)
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
                        AstType::llvm_ptr(context.state.mlir_context),
                        context.state,
                        &current,
                    )
                    .into_mlir();
                value.store(slot, context.state, &current);
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
                        let BlockAnd { value: values, block: next } = call.emit(context, current);
                        current = next;
                        values.into_iter().map(Some).collect()
                    } else {
                        let BlockAnd { value, block: next } = expression.emit(context, current);
                        current = next;
                        vec![Some(value)]
                    }
                } else {
                    (0..variables.len()).map(|_| None).collect()
                };
            let state = context.state;
            for (identifier, initial) in variables.iter().zip(initials) {
                let slot = YulValue::alloca(state, &current);
                let word =
                    initial.unwrap_or_else(|| YulValue::constant(&BigInt::from(0u32), state, &current));
                word.store(slot, state, &current);
                context.environment.define_variable(identifier.node_id(), slot);
            }
            Some(current)
        }
        YulStatement::YulExpression(expression) => {
            let BlockAnd { block: current, .. } = expression.emit(context, block);
            Some(current)
        }
        YulStatement::YulIfStatement(if_statement) => {
            let condition = if_statement.condition();
            let BlockAnd { value: condition_value, block } = condition.emit(context, block);
            let then_region = Region::new();
            then_region.append_block(Block::new(&[]));
            let else_region = Region::new();
            let operation = block.append_operation(mlir_op_build!(
                context.state,
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

            let (condition_block, body_block, step_block) = mlir_region_op!(
                context.state,
                &current,
                ForOperation.init_args(&[]).results(&[]);
                cond, body, step
            );

            let condition_region = condition_block
                .parent_region()
                .expect("yul.for cond block has a parent region");
            let saved_region = context.region_pointer;
            context.region_pointer = &*condition_region as *const _;
            let BlockAnd { value: condition_value, block: condition_end } = for_statement.condition().emit(context, condition_block);
            mlir_op_void!(
                context.state,
                &condition_end,
                ConditionOperation.condition(condition_value).args(&[])
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
            Some(block)
        }
        YulStatement::YulLeaveStatement(_) => {
            unimplemented!("a yul `leave` nested in control flow is not supported by the inliner")
        }
        YulStatement::YulBreakStatement(_) => {
            mlir_op_void!(context.state, &block, BreakOperation);
            None
        }
        YulStatement::YulContinueStatement(_) => {
            mlir_op_void!(context.state, &block, ContinueOperation);
            None
        }
        YulStatement::YulSwitchStatement(switch_statement) => {
            let BlockAnd { value: selector, block: current } = switch_statement.expression().emit(context, block);
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

            if value_cases.is_empty() {
                return match default_body {
                    Some(default_body) => default_body.emit(context, current),
                    None => Some(current),
                };
            }

            let context_handle = context.state.mlir_context;
            let case_attributes: Vec<Attribute<'context>> = value_cases
                .iter()
                .map(|case| {
                    AstType::signless(context_handle, solx_utils::BIT_LENGTH_FIELD)
                        .big_integer_attribute(&case.value().value())
                })
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
                context.state,
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
                    mlir_op_void!(context.state, &default_block, YieldOperation.operands(&[]));
                }
            }
            Some(current)
        }
    }
});
