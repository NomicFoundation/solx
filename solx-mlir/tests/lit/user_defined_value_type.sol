// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*id_u.*}}: ui256) -> ui256
// CHECK: sol.func @{{.*id_s.*}}: si8) -> si8
// CHECK: sol.func @{{.*id_a.*}}: !sol.address) -> !sol.address
// CHECK: sol.func @{{.*id_b.*}}: i1) -> i1

// `U.wrap` / `U.unwrap` are bit-level identities over the underlying `ui256`.
// CHECK: sol.func @{{.*do_wrap.*}}: ui256) -> ui256
// CHECK: sol.func @{{.*do_unwrap.*}}: ui256) -> ui256

contract C {
    type U is uint256;
    type S is int8;
    type A is address;
    type B is bool;

    function id_u(U x) public pure returns (U) { return x; }
    function id_s(S x) public pure returns (S) { return x; }
    function id_a(A x) public pure returns (A) { return x; }
    function id_b(B x) public pure returns (B) { return x; }

    function do_wrap(uint256 v) public pure returns (U) { return U.wrap(v); }
    function do_unwrap(U x) public pure returns (uint256) { return U.unwrap(x); }
}
