// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// delete on a storage array: solx emits one sol.delete deep-clear on the slot
// pointer; solc materializes a zero_init sol.malloc Memory buffer and sol.copy's
// it over storage.

// CHECK: sol.func @{{.*}}deleteArray
// CHECK-NEXT: sol.addr_of @array_{{[0-9]+}} : !sol.array<? x ui256, Storage>
// CHECK-SOLX-NEXT: sol.delete %{{.*}} : !sol.array<? x ui256, Storage>
// CHECK-SOLC-NEXT: sol.malloc zero_init : !sol.array<? x ui256, Memory>
// CHECK-SOLC-NEXT: sol.copy %{{.*}}, %{{.*}} : !sol.array<? x ui256, Memory>, !sol.array<? x ui256, Storage>

contract C {
    uint256[] array;
    uint256 x;
    mapping(uint256 => uint256) m;

    function deleteArray() public {
        delete array;
    }

    function deleteScalar() public {
        delete x;
    }

    function deleteMapElement() public {
        delete m[3];
    }
}
