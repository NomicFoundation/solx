// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*disordered.*}}
// CHECK:   sol.constant 10 : ui8
// CHECK:   sol.constant 3 : ui8
// CHECK:   sol.call @{{.*sub.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*named_struct.*}}
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.constant 2 : ui8

contract C {
    struct S {
        uint256 a;
        uint256 b;
    }

    function disordered() public pure returns (uint256) {
        return sub({y: 3, x: 10});
    }

    function named_struct() public pure returns (uint256) {
        S memory s = S({b: 2, a: 1});
        return s.a;
    }

    function sub(uint256 x, uint256 y) internal pure returns (uint256) {
        return x - y;
    }
}
