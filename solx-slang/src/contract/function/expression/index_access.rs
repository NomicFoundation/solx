//!
//! Index access expressions: fixed bytes, arrays, and mappings.
//!

use slang_solidity_v2::ast::IndexAccessExpression;
use slang_solidity_v2::ast::Type;

use solx_mlir::Place;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;
use solx_utils::DataLocation;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// `b[i]` on fixed bytes indexes the word directly; everything else loads from its place.
    pub fn index_access(&mut self, node: &IndexAccessExpression) -> Value<'context> {
        if node.is_slice() {
            return self.index_slice(node);
        }
        let base_type = node
            .operand()
            .get_type()
            .expect("base of index access has a resolved type");
        if matches!(base_type, Type::ByteArray(_)) {
            let base_value = self.expression(&node.operand());
            let index_expression = node
                .start()
                .expect("slang validates a[i] has an index expression");
            let index_value = self.expression(&index_expression);
            return base_value.fixed_bytes_index(index_value, self);
        }
        let (place, element_type) = self.index_access_place(node);
        let result_type = self.typing(node.get_type());
        place.load(element_type, self).bytes_cast(result_type, self)
    }

    /// The address yielded by `a[i]` / `m[k]` together with the element MLIR type.
    pub fn index_access_place(
        &mut self,
        node: &IndexAccessExpression,
    ) -> (Place<'context>, MlirType<'context>) {
        let base = node.operand();
        let base_type = base
            .get_type()
            .expect("base of index access has a resolved type");
        let base_value = self.expression(&base);
        let index_expression = node
            .start()
            .expect("slang validates a[i] has an index expression");
        let index_value = self.expression(&index_expression);

        match &base_type {
            Type::Mapping(_) => {
                let result_type = node
                    .get_type()
                    .expect("slang types every index-access expression");
                let element_type = self.resolve_type(&result_type, None);
                let base_location = match base_type.data_location() {
                    Some(location) => DataLocation::from_slang(location, None),
                    None => unimplemented!(
                        "index access on a value-typed base is not yet wired: {:?}",
                        std::mem::discriminant(&base_type)
                    ),
                };
                let address_type = self.pointer_type(&result_type, element_type, base_location);
                (
                    Place::from(base_value).map(index_value, address_type, self),
                    element_type,
                )
            }
            _ => {
                let element_type = base_value.r#type().element_type(0);
                (
                    Place::from(base_value).gep(index_value, element_type, self),
                    element_type,
                )
            }
        }
    }

    /// `a[start:end]` on a calldata array or `bytes`, defaulting an omitted `start` to zero and an
    /// omitted `end` to the length.
    fn index_slice(&mut self, node: &IndexAccessExpression) -> Value<'context> {
        let base = self.expression(&node.operand());
        let start = match node.start() {
            Some(start) => self.expression(&start),
            None => Value::zero(MlirType::field(self.melior), self),
        };
        let end = match node.end() {
            Some(end) => self.expression(&end),
            None => base.length(self),
        };
        base.slice(start, end, self)
    }
}
