// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*checked_add.*}}
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*checked_sub.*}}
// CHECK:   sol.csub %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*checked_mul.*}}
// CHECK:   sol.cmul %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*checked_div.*}}
// CHECK:   sol.cdiv %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*checked_mod.*}}
// CHECK:   sol.mod %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*checked_exp.*}}
// CHECK:   sol.cexp %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*unchecked_add.*}}
// CHECK:   sol.add %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*unchecked_sub.*}}
// CHECK:   sol.sub %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*unchecked_mul.*}}
// CHECK:   sol.mul %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*unchecked_div.*}}
// CHECK:   sol.div %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*unchecked_exp.*}}
// CHECK:   sol.exp %{{.*}}, %{{.*}} : ui256

// CHECK: sol.func @{{.*unary_neg.*}}
// CHECK:   %{{.*}} = sol.sub %{{.*}}, %{{.*}} : si256

// CHECK: sol.func @{{.*signed_add.*}}
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : si256

// CHECK: sol.func @{{.*signed_div.*}}
// CHECK:   sol.cdiv %{{.*}}, %{{.*}} : si256

// CHECK: sol.func @{{.*signed_shr.*}}
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
