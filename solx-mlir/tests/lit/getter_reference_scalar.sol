// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*data.*}}() -> !sol.string<Memory> attributes {{.*}}selector = 1943314746 : i32
// CHECK:   %[[P:.*]] = sol.addr_of @{{.*data.*}} : !sol.string<Storage>
// CHECK:   %[[C:.*]] = sol.data_loc_cast %[[P]] : !sol.string<Storage>, !sol.string<Memory>
// CHECK:   sol.return %[[C]] : !sol.string<Memory>
// CHECK: sol.func @{{.*name.*}}() -> !sol.string<Memory> attributes {{.*}}selector = 117300739 : i32
// CHECK:   %[[NP:.*]] = sol.addr_of @{{.*name.*}} : !sol.string<Storage>
// CHECK:   %[[NC:.*]] = sol.data_loc_cast %[[NP]] : !sol.string<Storage>, !sol.string<Memory>
// CHECK:   sol.return %[[NC]] : !sol.string<Memory>

contract C {
    bytes public data;
    string public name;
}
