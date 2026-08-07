// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}pushValue
// CHECK:   sol.push %{{.*}} : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func {{.*}}pushEmpty
// CHECK:   sol.push %{{.*}} : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>
// CHECK-NOT: sol.store

// CHECK: sol.func {{.*}}pushEmptyBraces
// CHECK:   sol.push %{{.*}} : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>
// CHECK-NOT: sol.store

// CHECK: sol.func {{.*}}pushAssign
// CHECK:   sol.push %{{.*}} : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func {{.*}}pushCompound
// CHECK:   sol.push %{{.*}} : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func {{.*}}pushStructMember
// CHECK:   sol.push %{{.*}} : !sol.array<? x !sol.struct<(ui256, ui256), Storage>, Storage> -> !sol.struct<(ui256, ui256), Storage>
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), Storage>, ui64, !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func {{.*}}pushStructValue
// CHECK:   sol.push %{{.*}} : !sol.array<? x !sol.struct<(ui256, ui256), Storage>, Storage> -> !sol.struct<(ui256, ui256), Storage>
// CHECK:   sol.copy %{{.*}}, %{{.*}} : !sol.struct<(ui256, ui256), Memory>, !sol.struct<(ui256, ui256), Storage>

// CHECK: sol.func {{.*}}pushNested
// CHECK:   sol.push %{{.*}} : !sol.array<? x !sol.array<? x ui256, Storage>, Storage> -> !sol.array<? x ui256, Storage>
// CHECK:   sol.push %{{.*}} : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func {{.*}}pushByte
// CHECK:   sol.push_string %{{.*}}, %{{.*}} : <Storage>, !sol.fixedbytes<1>

// CHECK: sol.func {{.*}}pushByteEmpty
// CHECK:   sol.push %{{.*}} : !sol.string<Storage> -> !sol.ptr<!sol.byte, Storage>

// CHECK: sol.func {{.*}}popLast
// CHECK:   sol.pop %{{.*}} : !sol.array<? x ui256, Storage>

// CHECK: sol.func {{.*}}popEmptyBraces
// CHECK:   sol.pop %{{.*}} : !sol.array<? x ui256, Storage>

// CHECK: sol.func {{.*}}popByte
// CHECK:   sol.pop %{{.*}} : !sol.string<Storage>

// CHECK: sol.func {{.*}}makeLiteral{{.*}}-> !sol.array<3 x ui256, Memory>
// CHECK:   sol.array_lit %{{.*}}, %{{.*}}, %{{.*}} : (ui256, ui256, ui256) -> !sol.array<3 x ui256, Memory>

contract C {
    struct Y {
        uint256 a;
        uint256 b;
    }

    uint256[] arr;
    bytes data;
    Y[] records;
    uint256[][] nested;

    function pushValue(uint256 x) public {
        arr.push(x);
    }

    function pushEmpty() public {
        arr.push();
    }

    function pushEmptyBraces() public {
        arr.push({});
    }

    function pushAssign(uint256 x) public {
        arr.push() = x;
    }

    function pushCompound(uint256 x) public {
        arr.push() += x;
    }

    function pushStructMember() public {
        records.push().a = 1;
    }

    function pushStructValue() public {
        records.push(Y(1, 2));
    }

    function pushNested() public {
        nested.push().push(2);
    }

    function pushByte(bytes1 element) public {
        data.push(element);
    }

    function pushByteEmpty() public {
        data.push();
    }

    function popLast() public {
        arr.pop();
    }

    function popEmptyBraces() public {
        arr.pop({});
    }

    function popByte() public {
        data.pop();
    }

    function makeLiteral(uint256 a, uint256 b, uint256 c) public pure returns (uint256[3] memory) {
        return [a, b, c];
    }
}
