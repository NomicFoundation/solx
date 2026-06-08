// RUN: solx --emit-mlir=sol %s | FileCheck %s

// `delete` clears an lvalue. A reference-typed storage aggregate uses the
// dedicated `sol.delete` deep-clear op (one op; its lowering recurses into
// elements/members); a value-typed lvalue (scalar, mapping element) is
// overwritten with zero. solc's nascent MLIR backend lowers the reference case
// as malloc-zero-init + copy instead, so this is a solx-only check; behavioural
// parity is covered by the tester.

// CHECK: sol.func @{{.*delArr.*}}
// CHECK: sol.delete %{{[0-9]+}} : !sol.array<? x ui256, Storage>

// CHECK: sol.func @{{.*delScalar.*}}
// CHECK: sol.store %{{.*}}, %{{[0-9]+}} : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func @{{.*delMapElem.*}}
// CHECK: sol.map
// CHECK: sol.store %{{.*}}, %{{[0-9]+}} : ui256, !sol.ptr<ui256, Storage>

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
