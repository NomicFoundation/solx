// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*choose.*}}
// CHECK: yul.switch
// CHECK: case 1 {
// CHECK: case 2 {
// CHECK: default {

// CHECK: sol.func @{{.*loop.*}}
// CHECK: yul.for cond {
// CHECK: yul.cmp ult
// CHECK: yul.condition
// CHECK: } body {
// CHECK: yul.if
// CHECK: yul.continue
// CHECK: yul.if
// CHECK: yul.break
// CHECK: } step {
// CHECK: yul.yield

// CHECK: sol.func @{{.*nested.*}}
// CHECK: yul.add
// CHECK: yul.mul

// CHECK: sol.func @{{.*no_default.*}}
// CHECK: yul.switch
// CHECK: case 1 {
// CHECK: case 2 {
// CHECK: default {

contract C {
    function choose(uint256 n) public pure returns (uint256 r) {
        assembly {
            switch n
            case 1 { r := 100 }
            case 2 { r := 200 }
            default { r := 300 }
        }
    }

    function loop(uint256 n) public pure returns (uint256 r) {
        assembly {
            for { let i := 0 } lt(i, n) { i := add(i, 1) } {
                if gt(i, 3) { continue }
                if gt(i, 9) { break }
                r := add(r, i)
            }
        }
    }

    function nested(uint256 a) public pure returns (uint256 r) {
        assembly {
            let x := a
            {
                let y := add(x, 1)
                { let z := mul(y, 2) r := z }
            }
        }
    }

    function no_default(uint256 n) public pure returns (uint256 r) {
        assembly {
            switch n
            case 1 { r := 10 }
            case 2 { r := 20 }
        }
    }
}
