// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A single-argument struct constructor `S(x)` is reported as a type conversion
// by slang (the struct name types as a metatype), but it must build the struct
// in memory (`sol.malloc` + `sol.gep` + `sol.store`) rather than cast its
// argument to the struct type (`sol.cast` is integer-only and rejects a struct
// result).

// CHECK: sol.func {{.*}}build{{.*}}-> !sol.struct<(ui256), Memory>
// CHECK:   sol.malloc :{{ +}}!sol.struct<(ui256), Memory>
// CHECK:   sol.constant 0 : ui64
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256), Memory>, ui64, !sol.ptr<ui256, Memory>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Memory>

contract C {
    struct S { uint256 a; }

    function build(uint256 x) public pure returns (S memory) {
        return S(x);
    }
}
