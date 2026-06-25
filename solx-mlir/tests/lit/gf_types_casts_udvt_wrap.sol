// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A user-defined value type resolves to its underlying representation type, so
// `wrap` / `unwrap` are representation no-ops: the function signatures carry the
// underlying type (si8 for `Amount is int8`, !sol.address for `Addr is
// address`) and the body just returns the value with no cast op. Both backends
// agree; function order differs so CHECK-DAG is used.

// CHECK-DAG: sol.func @{{.*w_s.*}}(%{{.*}}: si8) -> si8
// CHECK-DAG: sol.func @{{.*u_s.*}}(%{{.*}}: si8) -> si8
// CHECK-DAG: sol.func @{{.*w_a.*}}(%{{.*}}: !sol.address) -> !sol.address
// CHECK-DAG: sol.func @{{.*u_a.*}}(%{{.*}}: !sol.address) -> !sol.address

type Amount is int8;
type Addr is address;

contract C {
    function w_s(int8 x) public pure returns (Amount) { return Amount.wrap(x); }
    function u_s(Amount x) public pure returns (int8) { return Amount.unwrap(x); }
    function w_a(address x) public pure returns (Addr) { return Addr.wrap(x); }
    function u_a(Addr x) public pure returns (address) { return Addr.unwrap(x); }
}
