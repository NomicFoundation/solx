// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// Auto-generated getters for reference-type scalar state vars (`bytes`, `string`).
// Both backends `sol.addr_of` the storage slot. They DIVERGE on the return shape:
//   - solx returns the Storage reference directly (`-> !sol.string<Storage>`),
//     state_mutability #View.
//   - solc emits a `sol.data_loc_cast` Storage -> Memory and returns the Memory
//     reference (`-> !sol.string<Memory>`), state_mutability #NonPayable.
// Genuine benign divergence (loc-cast) -> split prefixes; same selectors.
// solx names them `data()` / `name()`, solc `get_data_<id>` / `get_name_<id>`.

// CHECK-SOLX: sol.func @{{.*data.*}}() -> !sol.string<Storage> attributes {{.*}}selector = 1943314746 : i32
// CHECK-SOLX:   %[[P:.*]] = sol.addr_of @{{.*data.*}} : !sol.string<Storage>
// CHECK-SOLX:   sol.return %[[P]] : !sol.string<Storage>
// CHECK-SOLX: sol.func @{{.*name.*}}() -> !sol.string<Storage> attributes {{.*}}selector = 117300739 : i32
// CHECK-SOLX:   %[[NP:.*]] = sol.addr_of @{{.*name.*}} : !sol.string<Storage>
// CHECK-SOLX:   sol.return %[[NP]] : !sol.string<Storage>

// CHECK-SOLC: sol.func @{{.*data.*}}() -> !sol.string<Memory> attributes {{.*}}selector = 1943314746 : i32
// CHECK-SOLC:   %[[P:.*]] = sol.addr_of @{{.*data.*}} : !sol.string<Storage>
// CHECK-SOLC:   %[[C:.*]] = sol.data_loc_cast %[[P]] : !sol.string<Storage>, !sol.string<Memory>
// CHECK-SOLC:   sol.return %[[C]] : !sol.string<Memory>

contract C {
    bytes public data;
    string public name;
}
