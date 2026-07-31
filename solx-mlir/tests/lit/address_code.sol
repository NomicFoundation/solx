// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*bytecode.*}}
// CHECK:   %[[CODE_RECEIVER:.*]] = sol.address_cast %{{.*}} : !sol.address<payable> to !sol.address
// CHECK:   sol.code %[[CODE_RECEIVER]] : !sol.address -> !sol.string<Memory>

// CHECK: sol.func @{{.*bytecode_hash.*}}
// CHECK:   %[[HASH_RECEIVER:.*]] = sol.address_cast %{{.*}} : !sol.address<payable> to !sol.address
// CHECK:   sol.code_hash %[[HASH_RECEIVER]] : !sol.address -> ui256

contract C {
    function bytecode(address payable a) public view returns (bytes memory) { return a.code; }

    function bytecode_hash(address payable a) public view returns (bytes32) { return a.codehash; }
}
