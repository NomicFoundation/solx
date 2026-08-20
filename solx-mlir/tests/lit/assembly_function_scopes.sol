// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solx-only: the C++ frontend cannot compile either case yet, so there is no second RUN
// line to share these checks with.
//   - sibling blocks each declaring `g`: both frontends flatten a block's Yul functions
//     into the one `sol.inline_asm` symbol table, and the C++ one keys them on the bare
//     source name, so it fails MLIR verification with "redefinition of symbol named 'g'".
//     Upstream solc compiles the same source. solx qualifies the symbol with the
//     definition's node id.
//   - `leave` in a for-initializer: solx stops emitting at a terminator, so the
//     unreachable loop is dropped; the C++ frontend splits the block and emits it.

// Yul scopes a function to its block, so two sibling blocks may each declare `g`. The two
// land in the same symbol table under distinct symbols, and each call resolves to its own.
// CHECK: sol.func @{{.*sibling_scopes.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @[[G1:.*]] : () -> i256 {
// CHECK:     yul.func @[[G2:.*]] : () -> i256 {
// CHECK:     yul.func_call @[[G1]]() : () -> i256
// CHECK:     yul.func_call @[[G2]]() : () -> i256

// A `leave` in the initializer terminates the block, so the loop never gets emitted.
// CHECK: sol.func @{{.*leave_in_for_init.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @{{.*g.*}} : () -> i256 {
// CHECK-NOT:   yul.for
// CHECK:       yul.func_return

contract C {
    function sibling_scopes() public pure returns (uint256 r) {
        assembly {
            { function g() -> y { y := 1 } r := g() }
            { function g() -> y { y := 2 } r := add(r, g()) }
        }
    }

    function leave_in_for_init() public pure returns (uint256 r) {
        assembly {
            function g() -> y { for { y := 1 leave } 1 {} {} }
            r := g()
        }
    }
}
