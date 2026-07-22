// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*add.*}}(%{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK:   sol.cadd

// CHECK: sol.func @{{.*double.*}}
// CHECK:   sol.call @{{.*add.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*chain.*}}
// CHECK:   sol.call @{{.*double.*}}
// CHECK:   sol.call @{{.*add.*}}

// CHECK: sol.func @{{.*literal_argument.*}}
// CHECK:   sol.constant 1633837924 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.call @{{.*literal_receiver.*}}

// CHECK: sol.func @{{.*widening_argument.*}}
// CHECK:   sol.load %{{.*}}
// CHECK:   sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.load %{{.*}}
// CHECK:   sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.call @{{.*add.*}}

// CHECK: sol.func @{{.*tuple_statement.*}}
// CHECK:   sol.call @{{.*add.*}}
// CHECK:   sol.call @{{.*double.*}}

contract C {
    function add(uint256 a, uint256 b) public pure returns (uint256) {
        return a + b;
    }

    function double(uint256 x) public pure returns (uint256) {
        return add(x, x);
    }

    function chain(uint256 x) public pure returns (uint256) {
        return add(double(x), x);
    }

    function literal_receiver(bytes4 selector) internal pure {}

    function literal_argument() public pure {
        literal_receiver("abcd");
    }

    function widening_argument(uint8 a, uint8 b) public pure returns (uint256) {
        return add(a, b);
    }

    function tuple_statement(uint256 x) public pure {
        (add(x, x), double(x));
    }
}
