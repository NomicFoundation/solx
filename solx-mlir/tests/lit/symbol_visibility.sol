// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=SOL
// RUN: solx --emit-mlir=llvm %s | FileCheck %s --check-prefix=KEPT
// RUN: solx --emit-mlir=llvm %s | FileCheck %s --check-prefix=DEAD

// An entry point - selector-dispatched, constructor, fallback, receive, getter - is a public
// symbol; everything else is private and symbol-dce removes it unless an entry point reaches
// it.

// SOL: sol.contract @{{.*:C"}} {
// SOL-DAG: sol.func @"@constructor()_{{[0-9]+}}"() attributes {kind = #Constructor
// SOL-DAG: sol.func @"pub(uint256)_{{[0-9]+}}"(%{{.*}}: ui256) -> ui256 attributes {{.*}}selector
// SOL-DAG: sol.func @"ext()_{{[0-9]+}}"() -> ui256 attributes {{.*}}selector
// SOL-DAG: sol.func @"v()_{{[0-9]+}}"() -> ui256 attributes {{.*}}selector
// SOL-DAG: sol.func @"fallback()_{{[0-9]+}}"() attributes {kind = #Fallback
// SOL-DAG: sol.func @"receive()_{{[0-9]+}}"() attributes {kind = #Receive
// SOL-DAG: sol.func private @"priv(uint256)_{{[0-9]+}}"
// SOL-DAG: sol.func private @"internal_dead(uint256)_{{[0-9]+}}"
// SOL-DAG: sol.func private @"private_dead()_{{[0-9]+}}"
// SOL-DAG: sol.func private @"li(uint256)_{{[0-9]+}}"
// SOL-DAG: sol.func private @"free(uint256)_{{[0-9]+}}"
// SOL: } {kind = #Contract}
// SOL: sol.contract @{{.*:L"}} {
// SOL-DAG: sol.func private @"li(uint256)_{{[0-9]+}}"
// SOL-DAG: sol.func private @"lu(uint256)_{{[0-9]+}}"
// SOL-DAG: sol.func @"lp(uint256)_{{[0-9]+}}"(%{{.*}}: ui256) -> ui256 attributes {{.*}}selector
// SOL: } {kind = #Library}

// KEPT: module @{{.*:C_deployed"}}
// KEPT-DAG: llvm.func private @"pub(uint256)_
// KEPT-DAG: llvm.func private @"ext()_
// KEPT-DAG: llvm.func private @"v()_
// KEPT-DAG: llvm.func private @"fallback()_
// KEPT-DAG: llvm.func private @"receive()_
// KEPT-DAG: llvm.func private @"priv(uint256)_
// KEPT-DAG: llvm.func private @"li(uint256)_
// KEPT-DAG: llvm.func private @"free(uint256)_
// KEPT: module @{{.*:D_deployed"}}
// KEPT: llvm.func private @"internal_live(uint256)_
// KEPT: "evm-unsafe-asm"
// KEPT: module @{{.*:L_deployed"}}
// KEPT: llvm.func private @"lp(uint256)_

// DEAD: module @{{.*:C_deployed"}}
// DEAD-NOT: @"internal_dead(
// DEAD-NOT: @"private_dead(
// DEAD-NOT: @"lu(
// DEAD-NOT: "evm-unsafe-asm"
// DEAD: module @{{.*:D"}}
// DEAD: module @{{.*:L"}}
// DEAD-NOT: @"li(
// DEAD-NOT: @"lu(

library L {
    function li(uint256 x) internal pure returns (uint256) { return x + 1; }
    function lu(uint256 x) internal pure returns (uint256) { return x + 2; }
    function lp(uint256 x) public pure returns (uint256) { return x + 3; }
}

function free(uint256 x) pure returns (uint256) { return x * 2; }

contract C {
    uint256 public v;

    modifier m() { require(v == 0); _; }

    constructor() {}

    function pub(uint256 x) public m returns (uint256) { return priv(x) + L.li(x) + free(x); }

    function ext() external pure returns (uint256) { return 1; }

    function priv(uint256 x) private pure returns (uint256) { return x; }

    function internal_dead(uint256 x) internal pure returns (uint256) {
        assembly { mstore(0, x) }
        return x;
    }

    function private_dead() private pure returns (uint256) { return 2; }

    function via_pointer() public pure returns (uint256) {
        function (uint256) pure returns (uint256) f = priv;
        return f(1);
    }

    fallback() external {}

    receive() external payable {}
}

contract D {
    function internal_live(uint256 x) internal pure returns (uint256) {
        assembly { mstore(0, x) }
        return x;
    }

    function run(uint256 x) public pure returns (uint256) { return internal_live(x); }
}
