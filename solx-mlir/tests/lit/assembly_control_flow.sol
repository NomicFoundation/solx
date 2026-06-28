// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Inline-assembly control flow lowers to the Yul-dialect structured ops:
// `yul.for` (cond/body/step regions terminated by `yul.condition`/`yul.yield`),
// `yul.if` (raw word condition, no else), `yul.break`/`yul.continue`, and
// `yul.switch` (one region per case plus a default).

// CHECK-DAG: sol.func @{{.*loop.*}}
// CHECK-DAG: yul.for cond {
// CHECK-DAG: yul.cmp ult
// CHECK-DAG: yul.condition
// CHECK-DAG: } body {
// CHECK-DAG: yul.if
// CHECK-DAG: yul.continue
// CHECK-DAG: yul.if
// CHECK-DAG: yul.break
// CHECK-DAG: } step {
// CHECK-DAG: yul.yield

// CHECK-DAG: sol.func @{{.*choose.*}}
// CHECK-DAG: yul.switch
// CHECK-DAG: case 1 {
// CHECK-DAG: case 2 {
// CHECK-DAG: default {

// A switch with no source default still lowers to yul.switch with a synthesized
// empty default region; nested plain blocks are flattened into the body.
// CHECK-DAG: sol.func @{{.*no_default.*}}
// CHECK-DAG: yul.switch
// CHECK-DAG: case 1 {
// CHECK-DAG: case 2 {
// CHECK-DAG: default {

// CHECK-DAG: sol.func @{{.*nested.*}}
// CHECK-DAG: yul.add
// CHECK-DAG: yul.mul

contract C {
    function loop(uint256 n) public pure returns (uint256 r) {
        assembly {
            for { let i := 0 } lt(i, n) { i := add(i, 1) } {
                if gt(i, 3) { continue }
                if gt(i, 9) { break }
                r := add(r, i)
            }
        }
    }

    function choose(uint256 n) public pure returns (uint256 r) {
        assembly {
            switch n
            case 1 { r := 100 }
            case 2 { r := 200 }
            default { r := 300 }
        }
    }

    function no_default(uint256 n) public pure returns (uint256 r) {
        assembly {
            switch n
            case 1 { r := 10 }
            case 2 { r := 20 }
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
}
