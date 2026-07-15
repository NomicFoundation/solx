// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*add_assign.*}}
// CHECK:   sol.store %arg0, %[[XPTR:.*]] :
// CHECK:   sol.store %arg1, %[[YPTR:.*]] :
// CHECK:   %[[RHS:.*]] = sol.load %[[YPTR]]
// CHECK:   %[[OLD:.*]] = sol.load %[[XPTR]]
// CHECK:   sol.cadd %[[OLD]], %[[RHS]]
// CHECK:   sol.store %{{.*}}, %[[XPTR]]

// CHECK: sol.func @{{.*sub_assign.*}}
// CHECK:   sol.store %arg0, %[[XPTR:.*]] :
// CHECK:   sol.store %arg1, %[[YPTR:.*]] :
// CHECK:   %[[RHS:.*]] = sol.load %[[YPTR]]
// CHECK:   %[[OLD:.*]] = sol.load %[[XPTR]]
// CHECK:   sol.csub %[[OLD]], %[[RHS]]
// CHECK:   sol.store %{{.*}}, %[[XPTR]]

// CHECK: sol.func @{{.*mul_assign.*}}
// CHECK:   sol.store %arg0, %[[XPTR:.*]] :
// CHECK:   sol.store %arg1, %[[YPTR:.*]] :
// CHECK:   %[[RHS:.*]] = sol.load %[[YPTR]]
// CHECK:   %[[OLD:.*]] = sol.load %[[XPTR]]
// CHECK:   sol.cmul %[[OLD]], %[[RHS]]
// CHECK:   sol.store %{{.*}}, %[[XPTR]]

// CHECK: sol.func @{{.*div_assign.*}}
// CHECK:   sol.store %arg0, %[[XPTR:.*]] :
// CHECK:   sol.store %arg1, %[[YPTR:.*]] :
// CHECK:   %[[RHS:.*]] = sol.load %[[YPTR]]
// CHECK:   %[[OLD:.*]] = sol.load %[[XPTR]]
// CHECK:   sol.cdiv %[[OLD]], %[[RHS]]
// CHECK:   sol.store %{{.*}}, %[[XPTR]]

// CHECK: sol.func @{{.*mod_assign.*}}
// CHECK:   sol.store %arg0, %[[XPTR:.*]] :
// CHECK:   sol.store %arg1, %[[YPTR:.*]] :
// CHECK:   %[[RHS:.*]] = sol.load %[[YPTR]]
// CHECK:   %[[OLD:.*]] = sol.load %[[XPTR]]
// CHECK:   sol.mod %[[OLD]], %[[RHS]]
// CHECK:   sol.store %{{.*}}, %[[XPTR]]

// CHECK: sol.func @{{.*and_assign.*}}
// CHECK:   sol.store %arg0, %[[XPTR:.*]] :
// CHECK:   sol.store %arg1, %[[YPTR:.*]] :
// CHECK:   %[[RHS:.*]] = sol.load %[[YPTR]]
// CHECK:   %[[OLD:.*]] = sol.load %[[XPTR]]
// CHECK:   sol.and %[[OLD]], %[[RHS]]
// CHECK:   sol.store %{{.*}}, %[[XPTR]]

// CHECK: sol.func @{{.*or_assign.*}}
// CHECK:   sol.store %arg0, %[[XPTR:.*]] :
// CHECK:   sol.store %arg1, %[[YPTR:.*]] :
// CHECK:   %[[RHS:.*]] = sol.load %[[YPTR]]
// CHECK:   %[[OLD:.*]] = sol.load %[[XPTR]]
// CHECK:   sol.or %[[OLD]], %[[RHS]]
// CHECK:   sol.store %{{.*}}, %[[XPTR]]

// CHECK: sol.func @{{.*xor_assign.*}}
// CHECK:   sol.store %arg0, %[[XPTR:.*]] :
// CHECK:   sol.store %arg1, %[[YPTR:.*]] :
// CHECK:   %[[RHS:.*]] = sol.load %[[YPTR]]
// CHECK:   %[[OLD:.*]] = sol.load %[[XPTR]]
// CHECK:   sol.xor %[[OLD]], %[[RHS]]
// CHECK:   sol.store %{{.*}}, %[[XPTR]]

// CHECK: sol.func @{{.*shl_assign.*}}
// CHECK:   sol.store %arg0, %[[XPTR:.*]] :
// CHECK:   sol.store %arg1, %[[YPTR:.*]] :
// CHECK:   %[[RHS:.*]] = sol.load %[[YPTR]]
// CHECK:   %[[OLD:.*]] = sol.load %[[XPTR]]
// CHECK:   sol.shl %[[OLD]], %[[RHS]]
// CHECK:   sol.store %{{.*}}, %[[XPTR]]

// CHECK: sol.func @{{.*shr_assign.*}}
// CHECK:   sol.store %arg0, %[[XPTR:.*]] :
// CHECK:   sol.store %arg1, %[[YPTR:.*]] :
// CHECK:   %[[RHS:.*]] = sol.load %[[YPTR]]
// CHECK:   %[[OLD:.*]] = sol.load %[[XPTR]]
// CHECK:   sol.shr %[[OLD]], %[[RHS]]
// CHECK:   sol.store %{{.*}}, %[[XPTR]]

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
