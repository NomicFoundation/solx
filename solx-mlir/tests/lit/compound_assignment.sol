// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*add_assign.*}}
// CHECK:   sol.store %arg0, %[[PTR:.*]] :
// CHECK:   sol.cadd
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @{{.*sub_assign.*}}
// CHECK:   sol.store %arg0, %[[PTR:.*]] :
// CHECK:   sol.csub
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @{{.*mul_assign.*}}
// CHECK:   sol.store %arg0, %[[PTR:.*]] :
// CHECK:   sol.cmul
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @{{.*div_assign.*}}
// CHECK:   sol.store %arg0, %[[PTR:.*]] :
// CHECK:   sol.cdiv
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @{{.*mod_assign.*}}
// CHECK:   sol.store %arg0, %[[PTR:.*]] :
// CHECK:   sol.mod
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @{{.*and_assign.*}}
// CHECK:   sol.store %arg0, %[[PTR:.*]] :
// CHECK:   sol.and
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @{{.*or_assign.*}}
// CHECK:   sol.store %arg0, %[[PTR:.*]] :
// CHECK:   sol.or
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @{{.*xor_assign.*}}
// CHECK:   sol.store %arg0, %[[PTR:.*]] :
// CHECK:   sol.xor
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @{{.*shl_assign.*}}
// CHECK:   sol.store %arg0, %[[PTR:.*]] :
// CHECK:   sol.shl
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @{{.*shr_assign.*}}
// CHECK:   sol.store %arg0, %[[PTR:.*]] :
// CHECK:   sol.shr
// CHECK:   sol.store %{{.*}}, %[[PTR]]

contract C {
    function add_assign(uint256 x, uint256 y) public pure returns (uint256) {
        x += y;
        return x;
    }

    function sub_assign(uint256 x, uint256 y) public pure returns (uint256) {
        x -= y;
        return x;
    }

    function mul_assign(uint256 x, uint256 y) public pure returns (uint256) {
        x *= y;
        return x;
    }

    function div_assign(uint256 x, uint256 y) public pure returns (uint256) {
        x /= y;
        return x;
    }

    function mod_assign(uint256 x, uint256 y) public pure returns (uint256) {
        x %= y;
        return x;
    }

    function and_assign(uint256 x, uint256 y) public pure returns (uint256) {
        x &= y;
        return x;
    }

    function or_assign(uint256 x, uint256 y) public pure returns (uint256) {
        x |= y;
        return x;
    }

    function xor_assign(uint256 x, uint256 y) public pure returns (uint256) {
        x ^= y;
        return x;
    }

    function shl_assign(uint256 x, uint256 y) public pure returns (uint256) {
        x <<= y;
        return x;
    }

    function shr_assign(uint256 x, uint256 y) public pure returns (uint256) {
        x >>= y;
        return x;
    }
}
