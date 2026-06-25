// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Yul control-flow constructs not exercised by assembly_control_flow.sol:
//  * a `switch` with no source `default` clause still lowers to `yul.switch`
//    with a synthesized empty `default` region in both frontends;
//  * nested plain Yul blocks are flattened, their inner statements emitting the
//    expected ops (here yul.add then yul.mul) in the enclosing function body.
// Functions are named a_/b_ so solx's alphabetical walk and solc's source-order
// walk produce the same CHECK-LABEL sequence.

// CHECK: sol.func @{{.*a_switch_nodefault.*}}
// CHECK: yul.switch
// CHECK: case 1 {
// CHECK: case 2 {
// CHECK: default {
// CHECK: sol.func @{{.*b_nested.*}}
// CHECK: yul.add
// CHECK: yul.mul

contract C {
    function a_switch_nodefault(uint256 n) public pure returns (uint256 r) {
        assembly {
            switch n
            case 1 { r := 10 }
            case 2 { r := 20 }
        }
    }
    function b_nested(uint256 a) public pure returns (uint256 r) {
        assembly {
            let x := a
            {
                let y := add(x, 1)
                { let z := mul(y, 2) r := z }
            }
        }
    }
}
