//!
//! The ternary conditional operator.
//!

use slang_solidity_v2::ast::ConditionalExpression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// The ternary conditional operator, lowered to a branch on the condition so only the selected
    /// arm runs. Each arm stores its element values into per-element stack slots, one for a scalar
    /// result and one per element for a tuple, loaded after the merge.
    pub fn conditional_values(&mut self, node: &ConditionalExpression) -> Vec<Value<'context>> {
        let element_types: Vec<Type<'context>> = match node.get_type() {
            Some(SlangType::Tuple(tuple_type)) => tuple_type
                .types()
                .iter()
                .map(|element_type| self.typing(Some(element_type.clone())))
                .collect(),
            Some(scalar) => vec![self.typing(Some(scalar))],
            // TODO: Slang does not resolve the type of a conditional whose arms differ in shape,
            // such as a call versus a tuple literal, though the common type is well-defined; remove
            // this arm once Slang v2 types such conditionals.
            None => unimplemented!(
                "conditional with heterogeneously-shaped arms: Slang does not resolve its type"
            ),
        };
        let condition = self.expression(&node.operand()).is_nonzero(self);
        let places: Vec<Place<'context>> = element_types
            .iter()
            .map(|&element_type| Place::stack(element_type, self))
            .collect();
        let (then_block, else_block) = self.current_block().branch_with_else(condition, self);
        for (block, branch) in [
            (then_block, node.true_expression()),
            (else_block, node.false_expression()),
        ] {
            self.region(block, |scope| {
                for ((place, &element_type), value) in places
                    .iter()
                    .zip(&element_types)
                    .zip(scope.expression_values(&branch))
                {
                    place.store(value.convert(element_type, scope), scope);
                }
            });
        }
        places
            .iter()
            .zip(&element_types)
            .map(|(place, &element_type)| place.load(element_type, self))
            .collect()
    }

    /// The ternary in single-value position: its sole value.
    pub fn conditional(&mut self, node: &ConditionalExpression) -> Value<'context> {
        self.conditional_values(node)
            .pop()
            .expect("a conditional yields at least one value")
    }

    /// The ternary in statement position, its selected arm run for side effects. Void arms carry no
    /// result type, so this cannot go through `conditional_values`.
    pub fn conditional_effect(&mut self, node: &ConditionalExpression) {
        let condition = self.expression(&node.operand()).is_nonzero(self);
        let (then_block, else_block) = self.current_block().branch_with_else(condition, self);
        self.region(then_block, |scope| {
            scope.expression_effect(&node.true_expression())
        });
        self.region(else_block, |scope| {
            scope.expression_effect(&node.false_expression())
        });
    }
}
