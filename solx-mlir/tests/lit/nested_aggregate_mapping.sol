// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*balances.*}} slot 0 offset 0 : !sol.mapping<!sol.address, !sol.mapping<ui256, ui256>>

// CHECK: sol.func {{.*}}read{{.*}}-> ui256
// CHECK:   sol.addr_of @{{.*balances.*}} : !sol.mapping<!sol.address, !sol.mapping<ui256, ui256>>
// CHECK:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<!sol.address, !sol.mapping<ui256, ui256>>, !sol.address, !sol.mapping<ui256, ui256>
// CHECK:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<ui256, ui256>, ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256

// CHECK: sol.func {{.*}}write
// CHECK:   sol.addr_of @{{.*balances.*}} : !sol.mapping<!sol.address, !sol.mapping<ui256, ui256>>
// CHECK:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<!sol.address, !sol.mapping<ui256, ui256>>, !sol.address, !sol.mapping<ui256, ui256>
// CHECK:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<ui256, ui256>, ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract C {
    mapping(address => mapping(uint256 => uint256)) balances;

    function read(address a, uint256 k) public view returns (uint256) {
        return balances[a][k];
    }

    function write(address a, uint256 k, uint256 v) public {
        balances[a][k] = v;
    }
}
