// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*LIMIT.*}}() -> ui256 attributes {{.*}}selector = -1350429457 : i32
// CHECK:   %[[C:.*]] = sol.constant 42 : ui8
// CHECK:   %[[V:.*]] = sol.cast %[[C]] : ui8 to ui256
// CHECK:   sol.return %[[V]] : ui256

// CHECK: sol.func @{{.*H.*}}() -> !sol.fixedbytes<32> attributes {{.*}}selector = 1889917025 : i32
// CHECK:   %[[S:.*]] = sol.string_lit "x" -> !sol.string<Memory>
// CHECK:   %[[H:.*]] = "sol.keccak256"(%[[S]]) : (!sol.string<Memory>) -> !sol.fixedbytes<32>
// CHECK:   sol.return %[[H]] : !sol.fixedbytes<32>

// CHECK: sol.func @{{.*value.*}}() -> ui256 attributes {{.*}}selector = 1067774533 : i32
// CHECK:   %[[P:.*]] = sol.addr_of @{{.*value.*}} : !sol.ptr<ui256, Storage>
// CHECK:   %[[V:.*]] = sol.load %[[P]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V]] : ui256

// CHECK: sol.func @{{.*owner.*}}() -> !sol.address attributes {{.*}}selector = -1918514341 : i32
// CHECK:   %[[OP:.*]] = sol.addr_of @{{.*owner.*}} : !sol.ptr<!sol.address, Storage>
// CHECK:   %[[OV:.*]] = sol.load %[[OP]] : !sol.ptr<!sol.address, Storage>, !sol.address
// CHECK:   sol.return %[[OV]] : !sol.address

// CHECK: sol.func @{{.*data.*}}() -> !sol.string<Memory> attributes {{.*}}selector = 1943314746 : i32
// CHECK:   %[[P:.*]] = sol.addr_of @{{.*data.*}} : !sol.string<Storage>
// CHECK:   %[[C:.*]] = sol.data_loc_cast %[[P]] : !sol.string<Storage>, !sol.string<Memory>
// CHECK:   sol.return %[[C]] : !sol.string<Memory>

// CHECK: sol.func @{{.*name.*}}() -> !sol.string<Memory> attributes {{.*}}selector = 117300739 : i32
// CHECK:   %[[NP:.*]] = sol.addr_of @{{.*name.*}} : !sol.string<Storage>
// CHECK:   %[[NC:.*]] = sol.data_loc_cast %[[NP]] : !sol.string<Storage>, !sol.string<Memory>
// CHECK:   sol.return %[[NC]] : !sol.string<Memory>

contract ConstantInt {
    uint256 public constant LIMIT = 42;
}

contract ConstantKeccak {
    bytes32 public constant H = keccak256("x");
}

contract Elementary {
    uint256 public value;
    address public owner;
}

contract Reference {
    bytes public data;
    string public name;
}
