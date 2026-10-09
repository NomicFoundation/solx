// RUN: slang --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.contract @{{.*:Entries"}} {
// CHECK-NOT: sol.func @
// CHECK: sol.func @"@constructor()_{{[0-9]+}}"() attributes {kind = #Constructor
// CHECK: sol.func @"shared_entry(uint256)_{{[0-9]+}}"(%{{.*}}: ui256) -> ui256
// CHECK-NOT: sol.func @
// CHECK: } {kind = #Contract}
// CHECK: sol.contract @{{.*:Entries"}} {
// CHECK-NOT: sol.func @
// CHECK: sol.func @"fallback()_{{[0-9]+}}"() attributes {kind = #Fallback
// CHECK-NOT: sol.func @
// CHECK: sol.func @"receive()_{{[0-9]+}}"() attributes {kind = #Receive
// CHECK-NOT: sol.func @
// CHECK: sol.func @"external_entry()_{{[0-9]+}}"() -> ui256 attributes {{.*}}selector
// CHECK-NOT: sol.func @
// CHECK: sol.func @"public_entry(uint256)_{{[0-9]+}}"(%{{.*}}: ui256) -> ui256 attributes {{.*}}selector
// CHECK-NOT: sol.func @
// CHECK: sol.func @"free_function(uint256)_{{[0-9]+}}"
// CHECK-NOT: sol.func @
// CHECK: sol.func @"library_internal_used(uint256)_{{[0-9]+}}"
// CHECK-NOT: sol.func @
// CHECK: sol.func @"private_live(uint256)_{{[0-9]+}}"
// CHECK-NOT: sol.func @
// CHECK: sol.func @"shared_entry(uint256)_{{[0-9]+}}"(%{{.*}}: ui256) -> ui256 attributes {{.*}}selector
// CHECK-NOT: sol.func @
// CHECK: sol.func @"via_pointer()_{{[0-9]+}}"() -> ui256 attributes {{.*}}selector
// CHECK: sol.func_constant @"pointer_target(uint256)_{{[0-9]+}}"
// CHECK-NOT: sol.func @
// CHECK: sol.func @"pointer_target(uint256)_{{[0-9]+}}"
// CHECK-NOT: sol.func @
// CHECK: sol.func @"counter()_{{[0-9]+}}"() -> ui256 attributes {{.*}}selector
// CHECK-NOT: sol.func @
// CHECK: } {kind = #Contract, runtime}
// CHECK: sol.contract @{{.*:Lib"}} {
// CHECK-NEXT: } {kind = #Library}
// CHECK: sol.contract @{{.*:Lib"}} {
// CHECK-NEXT: sol.func @"library_public(uint256)_{{[0-9]+}}"(%{{.*}}: ui256) -> ui256 attributes {{.*}}selector
// CHECK-NOT: sol.func @
// CHECK: } {kind = #Library, runtime}

library Lib {
    function library_internal_used(uint256 x) internal pure returns (uint256) { return x + 1; }
    function library_internal_unused(uint256 x) internal pure returns (uint256) { return x + 2; }
    function library_public(uint256 x) public pure returns (uint256) { return x + 3; }
}

function free_function(uint256 x) pure returns (uint256) { return x * 2; }

contract Entries {
    uint256 public counter;

    modifier when_zero() { require(counter == 0); _; }

    constructor() {
        shared_entry(0);
    }

    function public_entry(uint256 x) public when_zero returns (uint256) {
        return private_live(x) + Lib.library_internal_used(x) + free_function(x);
    }

    function external_entry() external pure returns (uint256) { return 1; }

    function shared_entry(uint256 x) public pure returns (uint256) { return x; }

    function private_live(uint256 x) private pure returns (uint256) { return x; }

    function internal_dead(uint256 x) internal pure returns (uint256) { return x; }

    function private_dead() private pure returns (uint256) { return 2; }

    function pointer_target(uint256 x) private pure returns (uint256) { return x + 4; }

    function via_pointer() public pure returns (uint256) {
        function (uint256) pure returns (uint256) f = pointer_target;
        return f(1);
    }

    fallback() external {}

    receive() external payable {}
}
