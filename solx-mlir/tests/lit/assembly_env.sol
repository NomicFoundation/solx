// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Environment / block-context and account-info Yul opcodes each lower to their
// own nullary/unary Yul-dialect op (rule 16). `blobbasefee` is intentionally
// excluded: solx does not yet implement YulBlobbasefee (see divergences).

// CHECK: sol.func @{{.*f.*}}
// CHECK: yul.caller
// CHECK: yul.origin
// CHECK: yul.coinbase
// CHECK: yul.timestamp
// CHECK: yul.number
// CHECK: yul.gaslimit
// CHECK: yul.chainid
// CHECK: yul.basefee
// CHECK: yul.gas
// CHECK: yul.gasprice
// CHECK: yul.callvalue
// CHECK: yul.selfbalance
// CHECK: yul.address
// CHECK: yul.balance
// CHECK: yul.extcodesize
// CHECK: yul.extcodehash

contract C {
    function f(address a) public view returns (uint256 r) {
        assembly {
            r := caller()
            r := origin()
            r := coinbase()
            r := timestamp()
            r := number()
            r := gaslimit()
            r := chainid()
            r := basefee()
            r := gas()
            r := gasprice()
            r := callvalue()
            r := selfbalance()
            r := address()
            r := balance(a)
            r := extcodesize(a)
            r := extcodehash(a)
        }
    }
}
