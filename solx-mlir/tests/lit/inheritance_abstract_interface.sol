// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.contract @{{.*Square.*}}
// CHECK:   sol.state_var @{{.*side.*}} slot 0 offset 0 : ui256
// CHECK:   sol.func @{{.*area.*}}() -> ui256 attributes {{.*}}selector = 1296140591
// CHECK:     sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256
// CHECK:     sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256
// CHECK:     sol.cmul
// CHECK:     sol.return
// CHECK:   sol.func @{{.*name.*}}() -> ui256 attributes {{.*}}selector = 117300739
// CHECK:     sol.constant 4 : ui8
// CHECK:     sol.return

interface IShape {
    function area() external view returns (uint256);
}

abstract contract Base is IShape {
    function name() public pure virtual returns (uint256);
}

contract Square is Base {
    uint256 side;

    function area() public view override returns (uint256) {
        return side * side;
    }

    function name() public pure override returns (uint256) {
        return 4;
    }
}
