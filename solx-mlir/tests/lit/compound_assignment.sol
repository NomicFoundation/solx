// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"add_assign(uint256,uint256)"
// CHECK:   %[[X:.*]] = sol.load %[[PTR:.*]] :
// CHECK:   sol.cadd %[[X]]
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @"sub_assign(uint256,uint256)"
// CHECK:   %[[X:.*]] = sol.load %[[PTR:.*]] :
// CHECK:   sol.csub %[[X]]
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @"mul_assign(uint256,uint256)"
// CHECK:   %[[X:.*]] = sol.load %[[PTR:.*]] :
// CHECK:   sol.cmul %[[X]]
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @"div_assign(uint256,uint256)"
// CHECK:   %[[X:.*]] = sol.load %[[PTR:.*]] :
// CHECK:   sol.cdiv %[[X]]
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @"mod_assign(uint256,uint256)"
// CHECK:   %[[X:.*]] = sol.load %[[PTR:.*]] :
// CHECK:   sol.mod %[[X]]
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @"and_assign(uint256,uint256)"
// CHECK:   %[[X:.*]] = sol.load %[[PTR:.*]] :
// CHECK:   sol.and %[[X]]
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @"or_assign(uint256,uint256)"
// CHECK:   %[[X:.*]] = sol.load %[[PTR:.*]] :
// CHECK:   sol.or %[[X]]
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @"xor_assign(uint256,uint256)"
// CHECK:   %[[X:.*]] = sol.load %[[PTR:.*]] :
// CHECK:   sol.xor %[[X]]
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @"shl_assign(uint256,uint256)"
// CHECK:   %[[X:.*]] = sol.load %[[PTR:.*]] :
// CHECK:   sol.shl %[[X]]
// CHECK:   sol.store %{{.*}}, %[[PTR]]

// CHECK: sol.func @"shr_assign(uint256,uint256)"
// CHECK:   %[[X:.*]] = sol.load %[[PTR:.*]] :
// CHECK:   sol.shr %[[X]]
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
