// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// `delete` on a value lvalue (scalar, mapping element) overwrites it with zero;
// both backends store ui256 0 through the storage pointer. On a reference-typed
// storage aggregate the lowering diverges: solx emits one `sol.delete` deep-clear
// op, while solc materializes a zero-init `sol.malloc` Memory buffer and `sol.copy`s
// it over storage. CHECK-SOLX pins the `sol.delete`; CHECK-SOLC pins the malloc/copy
// pair. The functions emit in a different order between the backends, so the shared
// checks are DAG.

// CHECK-DAG: sol.func @{{.*delArr.*}}
// CHECK-SOLX-DAG: sol.delete %{{.*}} : !sol.array<? x ui256, Storage>
// CHECK-SOLC-DAG: sol.malloc zero_init : !sol.array<? x ui256, Memory>
// CHECK-SOLC-DAG: sol.copy %{{.*}}, %{{.*}} : !sol.array<? x ui256, Memory>, !sol.array<? x ui256, Storage>
// CHECK-DAG: sol.func @{{.*delScalar.*}}
// CHECK-DAG: sol.func @{{.*delMapElem.*}}
// CHECK-DAG: sol.map %{{.*}}, %{{.*}} : !sol.mapping<ui256, ui256>, ui8, !sol.ptr<ui256, Storage>
// CHECK-DAG: sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK-DAG: sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256[] arr;
    uint256 x;
    mapping(uint256 => uint256) m;

    function delArr() public {
        delete arr;
    }

    function delScalar() public {
        delete x;
    }

    function delMapElem() public {
        delete m[3];
    }
}
