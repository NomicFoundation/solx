// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Copies land at their first reference, after the own functions; solc's print-init emits them
// first, so this is solx-only.

// CHECK: sol.contract @{{.*Lib.*}} {
// CHECK: sol.func @{{.*chain.*}}(%{{.*}}: ui256) -> ui256 attributes {id = {{[0-9]+}} : i64, state_mutability = #NonPayable}
// CHECK:   sol.call @{{.*link.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK: sol.func @{{.*link.*}}(%{{.*}}: ui256) -> ui256 attributes {id = {{[0-9]+}} : i64, state_mutability = #NonPayable}
// CHECK:   sol.revert "Overrun(uint256)" %{{.*}} : ui256 {call}
// CHECK:   sol.emit "Consumed(uint256)" indexed = [%{{.*}}] : ui256
// CHECK:   sol.malloc : !sol.struct<(ui256, ui256), Memory>
// CHECK: } {kind = #Library}

// CHECK: sol.contract @{{.*User.*}} {
// CHECK: sol.func @{{.*consume.*}}
// CHECK:   sol.call @{{.*chain.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK: sol.func @{{.*chain.*}}(%{{.*}}: ui256) -> ui256 attributes {id = {{[0-9]+}} : i64, state_mutability = #NonPayable}
// CHECK:   sol.call @{{.*link.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK: sol.func @{{.*link.*}}(%{{.*}}: ui256) -> ui256 attributes {id = {{[0-9]+}} : i64, state_mutability = #NonPayable}
// CHECK:   sol.revert "Overrun(uint256)" %{{.*}} : ui256 {call}
// CHECK:   sol.emit "Consumed(uint256)" indexed = [%{{.*}}] : ui256
// CHECK: } {kind = #Contract}

library Lib {
    enum Kind { Zero, One }

    error Overrun(uint256 x);

    event Consumed(uint256 indexed x);

    struct Pair {
        uint256 x;
        uint256 y;
    }

    function chain(uint256 a) internal returns (uint256) {
        return link(a) + 1;
    }

    function link(uint256 a) internal returns (uint256) {
        if (a == 0) revert Overrun(a);
        emit Consumed(a);
        Pair memory pair = Pair(a, uint256(Kind.One));
        return pair.x + pair.y;
    }
}

contract User {
    function consume(uint256 x) public returns (uint256) {
        return Lib.chain(x);
    }
}
