// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=SOL
// RUN: solx --emit-mlir=llvm %s | FileCheck %s --check-prefix=KEPT
// RUN: solx --emit-mlir=llvm %s | FileCheck %s --check-prefix=DEAD

// An entry point - selector-dispatched, constructor, fallback, receive, getter - is a public
// symbol; everything else is private and symbol-dce removes it unless an entry point reaches
// it.

// SOL: sol.contract @{{.*:Entries"}} {
// SOL-DAG: sol.func @"@constructor()_{{[0-9]+}}"() attributes {kind = #Constructor
// SOL-DAG: sol.func @"public_entry(uint256)_{{[0-9]+}}"(%{{.*}}: ui256) -> ui256 attributes {{.*}}selector
// SOL-DAG: sol.func @"external_entry()_{{[0-9]+}}"() -> ui256 attributes {{.*}}selector
// SOL-DAG: sol.func @"counter()_{{[0-9]+}}"() -> ui256 attributes {{.*}}selector
// SOL-DAG: sol.func @"via_pointer()_{{[0-9]+}}"() -> ui256 attributes {{.*}}selector
// SOL-DAG: sol.func_constant @"pointer_target(uint256)_{{[0-9]+}}"
// SOL-DAG: sol.func @"fallback()_{{[0-9]+}}"() attributes {kind = #Fallback
// SOL-DAG: sol.func @"receive()_{{[0-9]+}}"() attributes {kind = #Receive
// SOL-DAG: sol.func private @"private_live(uint256)_{{[0-9]+}}"
// SOL-DAG: sol.func private @"pointer_target(uint256)_{{[0-9]+}}"
// SOL-DAG: sol.func private @"internal_dead(uint256)_{{[0-9]+}}"
// SOL-DAG: sol.func private @"private_dead()_{{[0-9]+}}"
// SOL-DAG: sol.func private @"library_internal_used(uint256)_{{[0-9]+}}"
// SOL-DAG: sol.func private @"free_function(uint256)_{{[0-9]+}}"
// SOL: } {kind = #Contract}
// SOL: sol.contract @{{.*:Lib"}} {
// SOL-DAG: sol.func private @"library_internal_used(uint256)_{{[0-9]+}}"
// SOL-DAG: sol.func private @"library_internal_unused(uint256)_{{[0-9]+}}"
// SOL-DAG: sol.func @"library_public(uint256)_{{[0-9]+}}"(%{{.*}}: ui256) -> ui256 attributes {{.*}}selector
// SOL: } {kind = #Library}

// KEPT: module @{{.*:Entries_deployed"}}
// KEPT-DAG: llvm.func private @"public_entry(uint256)_
// KEPT-DAG: llvm.func private @"external_entry()_
// KEPT-DAG: llvm.func private @"counter()_
// KEPT-DAG: llvm.func private @"via_pointer()_
// KEPT-DAG: llvm.func private @"fallback()_
// KEPT-DAG: llvm.func private @"receive()_
// KEPT-DAG: llvm.func private @"private_live(uint256)_
// KEPT-DAG: llvm.func private @"pointer_target(uint256)_
// KEPT-DAG: llvm.func private @"library_internal_used(uint256)_
// KEPT-DAG: llvm.func private @"free_function(uint256)_
// KEPT: module @{{.*:Lib"}}
// KEPT: module @{{.*:Lib_deployed"}}
// KEPT: llvm.func private @"library_public(uint256)_
// KEPT: module @{{.*:Unsafe_deployed"}}
// KEPT: llvm.func private @"internal_live(uint256)_
// KEPT-NOT: "evm-memory-guard"

// DEAD: module @{{.*:Entries_deployed"}}
// DEAD-NOT: @"internal_dead(
// DEAD-NOT: @"private_dead(
// DEAD-NOT: @"library_internal_unused(
// DEAD: "evm-memory-guard"
// DEAD: module @{{.*:Lib"}}
// DEAD-NOT: @"library_internal_used(
// DEAD-NOT: @"library_internal_unused(
// DEAD: module @{{.*:Unsafe"}}

library Lib {
    function library_internal_used(uint256 x) internal pure returns (uint256) { return x + 1; }
    function library_internal_unused(uint256 x) internal pure returns (uint256) { return x + 2; }
    function library_public(uint256 x) public pure returns (uint256) { return x + 3; }
}

function free_function(uint256 x) pure returns (uint256) { return x * 2; }

contract Entries {
    uint256 public counter;

    modifier when_zero() { require(counter == 0); _; }

    constructor() {}

    function public_entry(uint256 x) public when_zero returns (uint256) {
        return private_live(x) + Lib.library_internal_used(x) + free_function(x);
    }

    function external_entry() external pure returns (uint256) { return 1; }

    function private_live(uint256 x) private pure returns (uint256) { return x; }

    function internal_dead(uint256 x) internal pure returns (uint256) {
        assembly { mstore(0, x) }
        return x;
    }

    function private_dead() private pure returns (uint256) { return 2; }

    function pointer_target(uint256 x) private pure returns (uint256) { return x + 4; }

    function via_pointer() public pure returns (uint256) {
        function (uint256) pure returns (uint256) f = pointer_target;
        return f(1);
    }

    fallback() external {}

    receive() external payable {}
}

contract Unsafe {
    function internal_live(uint256 x) internal pure returns (uint256) {
        assembly { mstore(0, x) }
        return x;
    }

    function run(uint256 x) public pure returns (uint256) { return internal_live(x); }
}
