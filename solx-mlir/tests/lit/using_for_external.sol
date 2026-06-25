// RUN: solx --emit-mlir=sol %s | FileCheck %s

// `using L for uint256` where the attached library function `add` is external
// (selector-bearing). The method-style call `x.add(3)` delegatecalls into the
// deployed library: the receiver `x` becomes the implicit leading `self`
// argument of the `sol.ext_call` (so the call type is `(ui256, ui256)`), the
// address is a `sol.lib_addr` link placeholder, and the op carries the
// `delegate_call` + `library_call` flags. solx-only: solc's MLIR frontend hits
// `SolidityToMLIR.cpp` NYI/UNREACHABLE on a `using for` external library call,
// so there is no `solc` RUN line to cross-check.

// CHECK-LABEL: sol.func @{{.*}}f
// CHECK: %[[X:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK: sol.lib_addr "{{.*}}:L" : !sol.address
// CHECK: sol.ext_call "add(uint256,uint256)"(%[[X]], %{{.*}}){{.*}}{{{.*}}delegate_call{{.*}}library_call{{.*}}}

library L {
    function add(uint256 a, uint256 b) external returns (uint256) {
        return a + b;
    }
}

contract C {
    using L for uint256;

    function f(uint256 x) external returns (uint256) {
        return x.add(3);
    }
}
