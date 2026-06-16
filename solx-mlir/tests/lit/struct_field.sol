// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.state_var @{{.*}} slot 0 offset 0 : !sol.struct<(ui256, ui256), Storage>

// CHECK-DAG: sol.func {{.*}}readField{{.*}}-> ui256
// CHECK-DAG:   sol.constant 1 : ui64
// CHECK-DAG:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), Memory>, ui64, !sol.ptr<ui256, Memory>
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, Memory>, ui256

// CHECK-DAG: sol.func {{.*}}readNested{{.*}}-> ui256
// CHECK-DAG:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(!sol.struct<(ui256, ui256), Memory>, ui256), Memory>, ui64, !sol.ptr<!sol.struct<(ui256, ui256), Memory>, Memory>
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<!sol.struct<(ui256, ui256), Memory>, Memory>, !sol.struct<(ui256, ui256), Memory>
// CHECK-DAG:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), Memory>, ui64, !sol.ptr<ui256, Memory>
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, Memory>, ui256

// CHECK-DAG: sol.func {{.*}}readCalldata{{.*}}-> ui256
// CHECK-DAG:   sol.constant 1 : ui64
// CHECK-DAG:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), CallData>, ui64, !sol.ptr<ui256, CallData>
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, CallData>, ui256

// CHECK-DAG: sol.func {{.*}}readStorage{{.*}}-> ui256
// CHECK-DAG:   sol.addr_of @{{.*}} : !sol.struct<(ui256, ui256), Storage>
// CHECK-DAG:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256

// CHECK-DAG: sol.func {{.*}}writeStorage
// CHECK-DAG:   sol.addr_of @{{.*}} : !sol.struct<(ui256, ui256), Storage>
// CHECK-DAG:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract C {
    struct Inner { uint256 a; uint256 b; }
    struct Outer { Inner inner; uint256 extra; }
    Inner data;

    function readField(Inner memory s) public pure returns (uint256) {
        return s.b;
    }

    function readNested(Outer memory o) public pure returns (uint256) {
        return o.inner.b;
    }

    function readCalldata(Inner calldata s) external pure returns (uint256) {
        return s.b;
    }

    function readStorage() public view returns (uint256) {
        return data.b;
    }

    function writeStorage(uint256 v) public {
        data.b = v;
    }
}
