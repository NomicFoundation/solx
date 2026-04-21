// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"checked_add(uint256,uint256)"
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"checked_sub(uint256,uint256)"
// CHECK:   sol.csub %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"checked_mul(uint256,uint256)"
// CHECK:   sol.cmul %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"checked_div(uint256,uint256)"
// CHECK:   sol.cdiv %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"checked_mod(uint256,uint256)"
// CHECK:   sol.mod %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"checked_exp(uint256,uint256)"
// CHECK:   sol.cexp %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"unchecked_add(uint256,uint256)"
// CHECK:   sol.add %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"unchecked_sub(uint256,uint256)"
// CHECK:   sol.sub %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"unchecked_mul(uint256,uint256)"
// CHECK:   sol.mul %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"unchecked_div(uint256,uint256)"
// CHECK:   sol.div %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"unchecked_exp(uint256,uint256)"
// CHECK:   sol.exp %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @"unary_neg(int256)"
// CHECK:   %{{.*}} = sol.sub %{{.*}}, %{{.*}} : si256

// CHECK: sol.func @"signed_add(int256,int256)"
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : si256

// CHECK: sol.func @"signed_div(int256,int256)"
// CHECK:   sol.cdiv %{{.*}}, %{{.*}} : si256

// CHECK: sol.func @"signed_shr(int256,uint256)"
// CHECK:   sol.shr %{{.*}}, %{{.*}} : si256

contract C {
    function checked_add(uint256 a, uint256 b) public pure returns (uint256) {
        return a + b;
    }

    function checked_sub(uint256 a, uint256 b) public pure returns (uint256) {
        return a - b;
    }

    function checked_mul(uint256 a, uint256 b) public pure returns (uint256) {
        return a * b;
    }

    function checked_div(uint256 a, uint256 b) public pure returns (uint256) {
        return a / b;
    }

    function checked_mod(uint256 a, uint256 b) public pure returns (uint256) {
        return a % b;
    }

    function checked_exp(uint256 a, uint256 b) public pure returns (uint256) {
        return a ** b;
    }

    function unchecked_add(uint256 a, uint256 b) public pure returns (uint256) {
        unchecked { return a + b; }
    }

    function unchecked_sub(uint256 a, uint256 b) public pure returns (uint256) {
        unchecked { return a - b; }
    }

    function unchecked_mul(uint256 a, uint256 b) public pure returns (uint256) {
        unchecked { return a * b; }
    }

    function unchecked_div(uint256 a, uint256 b) public pure returns (uint256) {
        unchecked { return a / b; }
    }

    function unchecked_exp(uint256 a, uint256 b) public pure returns (uint256) {
        unchecked { return a ** b; }
    }

    function unary_neg(int256 a) public pure returns (int256) {
        unchecked {
            return -a;
        }
    }

    function signed_add(int256 a, int256 b) public pure returns (int256) {
        return a + b;
    }

    function signed_div(int256 a, int256 b) public pure returns (int256) {
        return a / b;
    }

    function signed_shr(int256 a, uint256 b) public pure returns (int256) {
        return a >> b;
    }
}
