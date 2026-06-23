// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.func @{{.*add_assign.*}}
// CHECK-DAG:   sol.store %arg0, %[[PTR:.*]] :
// CHECK-DAG:   sol.cadd
// CHECK-DAG:   sol.store %{{.*}}, %[[PTR]]

// CHECK-DAG: sol.func @{{.*sub_assign.*}}
// CHECK-DAG:   sol.store %arg0, %[[PTR:.*]] :
// CHECK-DAG:   sol.csub
// CHECK-DAG:   sol.store %{{.*}}, %[[PTR]]

// CHECK-DAG: sol.func @{{.*mul_assign.*}}
// CHECK-DAG:   sol.store %arg0, %[[PTR:.*]] :
// CHECK-DAG:   sol.cmul
// CHECK-DAG:   sol.store %{{.*}}, %[[PTR]]

// CHECK-DAG: sol.func @{{.*div_assign.*}}
// CHECK-DAG:   sol.store %arg0, %[[PTR:.*]] :
// CHECK-DAG:   sol.cdiv
// CHECK-DAG:   sol.store %{{.*}}, %[[PTR]]

// CHECK-DAG: sol.func @{{.*mod_assign.*}}
// CHECK-DAG:   sol.store %arg0, %[[PTR:.*]] :
// CHECK-DAG:   sol.mod
// CHECK-DAG:   sol.store %{{.*}}, %[[PTR]]

// CHECK-DAG: sol.func @{{.*and_assign.*}}
// CHECK-DAG:   sol.store %arg0, %[[PTR:.*]] :
// CHECK-DAG:   sol.and
// CHECK-DAG:   sol.store %{{.*}}, %[[PTR]]

// CHECK-DAG: sol.func @{{.*or_assign.*}}
// CHECK-DAG:   sol.store %arg0, %[[PTR:.*]] :
// CHECK-DAG:   sol.or
// CHECK-DAG:   sol.store %{{.*}}, %[[PTR]]

// CHECK-DAG: sol.func @{{.*xor_assign.*}}
// CHECK-DAG:   sol.store %arg0, %[[PTR:.*]] :
// CHECK-DAG:   sol.xor
// CHECK-DAG:   sol.store %{{.*}}, %[[PTR]]

// CHECK-DAG: sol.func @{{.*shl_assign.*}}
// CHECK-DAG:   sol.store %arg0, %[[PTR:.*]] :
// CHECK-DAG:   sol.shl
// CHECK-DAG:   sol.store %{{.*}}, %[[PTR]]

// CHECK-DAG: sol.func @{{.*shr_assign.*}}
// CHECK-DAG:   sol.store %arg0, %[[PTR:.*]] :
// CHECK-DAG:   sol.shr
// CHECK-DAG:   sol.store %{{.*}}, %[[PTR]]

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
