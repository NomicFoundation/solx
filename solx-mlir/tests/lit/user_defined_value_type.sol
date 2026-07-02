// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*id_a.*}}: !sol.address) -> !sol.address
// CHECK: sol.func @{{.*id_b.*}}: i1) -> i1
// CHECK: sol.func @{{.*id_s.*}}: si8) -> si8
// CHECK: sol.func @{{.*id_u.*}}: ui256) -> ui256

// CHECK: sol.func @{{"?unwrap_a.*}}: !sol.address) -> !sol.address
// CHECK: sol.func @{{"?unwrap_s.*}}: si8) -> si8
// CHECK: sol.func @{{"?unwrap_u.*}}: ui256) -> ui256
// CHECK: sol.func @{{"?wrap_a.*}}: !sol.address) -> !sol.address
// CHECK: sol.func @{{"?wrap_s.*}}: si8) -> si8
// CHECK: sol.func @{{"?wrap_u.*}}: ui256) -> ui256

contract C {
    type U is uint256;
    type S is int8;
    type A is address;
    type B is bool;

    function id_a(A x) public pure returns (A) { return x; }

    function id_b(B x) public pure returns (B) { return x; }

    function id_s(S x) public pure returns (S) { return x; }

    function id_u(U x) public pure returns (U) { return x; }

    function unwrap_a(A x) public pure returns (address) { return A.unwrap(x); }

    function unwrap_s(S x) public pure returns (int8) { return S.unwrap(x); }

    function unwrap_u(U x) public pure returns (uint256) { return U.unwrap(x); }

    function wrap_a(address x) public pure returns (A) { return A.wrap(x); }

    function wrap_s(int8 x) public pure returns (S) { return S.wrap(x); }

    function wrap_u(uint256 x) public pure returns (U) { return U.wrap(x); }
}
