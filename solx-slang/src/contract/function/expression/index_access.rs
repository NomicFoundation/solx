//!
//! Index access expressions: fixed bytes, arrays, and mappings.
//!

use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Place;
use solx_utils::DataLocation;

use crate::contract::function::expression::Expression;
use crate::r#type::Type;

codegen!(
    IndexAccessExpression {
        /// `b[i]` on fixed bytes indexes the word directly; everything else loads from its place.
        -> Value |node, scope| {
            let base_type = node
                .operand()
                .get_type()
                .expect("base of index access has a resolved type");
            if matches!(base_type, SlangType::ByteArray(_)) {
                let base_value = Expression::emit(&node.operand(), scope);
                let index_expression = node
                    .start()
                    .expect("slang validates a[i] has an index expression");
                let index_value = Expression::emit(&index_expression, scope);
                return base_value.fixed_bytes_index(index_value, scope);
            }
            let (place, element_type) = Self::emit_place(node, scope);
            let result_type = codegen!(@result_type IndexAccessExpression, node, scope);
            place
                .load(element_type, scope)
                .bytes_cast(result_type, scope)
        }

        /// The address yielded by `a[i]` / `m[k]` together with the element MLIR type.
        -> Place |node, scope| {
            if node.end().is_some() {
                unimplemented!("range index (a[i:j]) is not yet supported");
            }

            let base = node.operand();
            let base_type = base
                .get_type()
                .expect("base of index access has a resolved type");
            let base_value = Expression::emit(&base, scope);
            let index_expression = node
                .start()
                .expect("slang validates a[i] has an index expression");
            let index_value = Expression::emit(&index_expression, scope);

            match &base_type {
                SlangType::Mapping(_) => {
                    let result_type = node
                        .get_type()
                        .expect("slang types every index-access expression");
                    let element_type = Type::resolve(&result_type, None, scope);
                    let base_location = match base_type.data_location() {
                        Some(location) => DataLocation::from_slang(location, None),
                        None => unimplemented!(
                            "index access on a value-typed base is not yet wired: {:?}",
                            std::mem::discriminant(&base_type)
                        ),
                    };
                    let address_type =
                        Type::address_type(&result_type, element_type, base_location, scope);
                    (
                        Place::from(base_value).map(index_value, address_type, scope),
                        element_type,
                    )
                }
                _ => {
                    let element_type = base_value.r#type().element_type(0);
                    (
                        Place::from(base_value).gep(index_value, element_type, scope),
                        element_type,
                    )
                }
            }
        }
    }
);
