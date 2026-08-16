// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*fold.*}}(%{{.*}}: !sol.struct<(ui256, ui256), Memory>) -> ui256 attributes {id = {{[0-9]+}} : i64, orig_fn_type = (!sol.struct<(ui256, ui256), Memory>) -> ui256, selector = -699220919 : i32, state_mutability = #Pure}

// CHECK: sol.func @{{.*grow.*}}(%{{.*}}: !sol.array<? x ui256, Storage>) -> ui256 attributes {id = {{[0-9]+}} : i64, orig_fn_type = (!sol.array<? x ui256, Storage>) -> ui256, selector = 1149502785 : i32, state_mutability = #NonPayable}
// CHECK:   sol.push %{{.*}} : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>

// CHECK: sol.func @{{.*error_selector.*}}
// CHECK:   sol.constant 219656568 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>

// CHECK: sol.func @{{.*function_selector.*}}
// CHECK:   sol.constant 2825669559 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>

library Lib {
    struct Pair {
        uint256 x;
        uint256 y;
    }

    error Halt(uint256 x);

    function fold(Pair memory pair) external pure returns (uint256) {
        return pair.x + pair.y;
    }

    function grow(uint256[] storage a) external returns (uint256) {
        a.push(1);
        return a.length;
    }

    function visible(uint256 a) external pure returns (uint256) {
        return a;
    }
}

contract User {
    function error_selector() external pure returns (bytes4) {
        return Lib.Halt.selector;
    }

    function function_selector() external pure returns (bytes4) {
        return Lib.visible.selector;
    }
}
