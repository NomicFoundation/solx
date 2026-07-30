// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*identity_unsigned.*}}: ui256) -> ui256
// CHECK: sol.func @{{.*identity_signed.*}}: si8) -> si8
// CHECK: sol.func @{{.*identity_address.*}}: !sol.address) -> !sol.address
// CHECK: sol.func @{{.*identity_boolean.*}}: i1) -> i1

// CHECK: sol.func @{{.*wrap_unsigned.*}}: ui256) -> ui256
// CHECK-NOT: _cast
// CHECK: sol.func @{{.*unwrap_unsigned.*}}: ui256) -> ui256
// CHECK-NOT: _cast
// CHECK: sol.func @{{.*wrap_signed.*}}: si8) -> si8
// CHECK-NOT: _cast
// CHECK: sol.func @{{.*unwrap_signed.*}}: si8) -> si8
// CHECK-NOT: _cast
// CHECK: sol.func @{{.*wrap_address.*}}: !sol.address) -> !sol.address
// CHECK-NOT: _cast
// CHECK: sol.func @{{.*unwrap_address.*}}: !sol.address) -> !sol.address
// CHECK-NOT: _cast
// CHECK: sol.func @{{.*wrap_boolean.*}}: i1) -> i1
// CHECK-NOT: _cast
// CHECK: sol.func @{{.*unwrap_boolean.*}}: i1) -> i1
// CHECK-NOT: _cast
// CHECK: sol.return

contract C {
    type Unsigned is uint256;
    type Signed is int8;
    type Address is address;
    type Boolean is bool;

    function identity_unsigned(Unsigned x) public pure returns (Unsigned) { return x; }
    function identity_signed(Signed x) public pure returns (Signed) { return x; }
    function identity_address(Address x) public pure returns (Address) { return x; }
    function identity_boolean(Boolean x) public pure returns (Boolean) { return x; }

    function wrap_unsigned(uint256 x) public pure returns (Unsigned) { return Unsigned.wrap(x); }
    function unwrap_unsigned(Unsigned x) public pure returns (uint256) { return Unsigned.unwrap(x); }
    function wrap_signed(int8 x) public pure returns (Signed) { return Signed.wrap(x); }
    function unwrap_signed(Signed x) public pure returns (int8) { return Signed.unwrap(x); }
    function wrap_address(address x) public pure returns (Address) { return Address.wrap(x); }
    function unwrap_address(Address x) public pure returns (address) { return Address.unwrap(x); }
    function wrap_boolean(bool x) public pure returns (Boolean) { return Boolean.wrap(x); }
    function unwrap_boolean(Boolean x) public pure returns (bool) { return Boolean.unwrap(x); }
}
