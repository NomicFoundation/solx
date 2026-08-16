// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc's print-init drops declaration-site immutable initializers while its legacy pipeline
// emits them, so solx follows legacy and this is solx-only.

// CHECK: sol.immutable @{{.*x.*}} : ui256

// CHECK: sol.func @{{.*}}() attributes {{.*}}kind = #{{.*}}Constructor
// CHECK:   %[[XP:.*]] = sol.addr_of @{{.*x.*}} : !sol.ptr<ui256, Immutable>
// CHECK:   sol.constant 42 : ui8
// CHECK:   sol.store %{{.*}}, %[[XP]] : ui256, !sol.ptr<ui256, Immutable>

// CHECK: sol.func @{{.*get.*}}() -> ui256 attributes {{.*}}selector = 1833756220 : i32
// CHECK:   %[[V:.*]] = sol.load_immutable @{{.*x.*}} : ui256
// CHECK:   sol.return %[[V]] : ui256

contract Initialized {
    uint256 immutable x = 42;

    function get() public view returns (uint256) {
        return x;
    }
}
