// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.contract @{{.*}}C{{.*}} {
// CHECK:   sol.func @{{.*use.*}}
// CHECK:   sol.func @{{.*twice.*}}
// CHECK:     sol.inline_asm {
// CHECK:       yul.mul %{{.*}}, %c2_i256
// CHECK: } {kind = #Contract}
// CHECK: sol.contract @{{.*}}L{{.*}} {
// CHECK:   sol.func @{{.*twice.*}}
// CHECK:     sol.inline_asm {
// CHECK:       yul.mul %{{.*}}, %c2_i256
// CHECK: } {kind = #Library}

library L {
    function twice(uint256 x) internal pure returns (uint256 r) {
        assembly {
            r := mul(x, 2)
        }
    }
}

contract C {
    function use(uint256 x) public pure returns (uint256 r) {
        r = L.twice(x);
    }
}
