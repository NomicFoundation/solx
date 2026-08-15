// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Copies inside the consumer module: solc's print-init drops the consumer module when a bare
// public sibling is reached, so this is solx-only.

// CHECK: sol.contract @{{.*User.*}} {
// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #Constructor
// CHECK:   sol.call @{{.*seed.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK: sol.func @{{.*seed.*}}(%{{.*}}: ui256) -> ui256 attributes {id = {{[0-9]+}} : i64, state_mutability = #Pure}
// CHECK:   sol.call @{{.*step.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK: sol.func @{{.*consume.*}}
// CHECK:   sol.call @{{.*calls_public_sibling.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK: sol.func @{{.*calls_public_sibling.*}}(%{{.*}}: ui256) -> ui256 attributes {id = {{[0-9]+}} : i64, state_mutability = #Pure}
// CHECK:   sol.call @{{.*public_sibling.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK: sol.func @{{.*public_sibling.*}}(%{{.*}}: ui256) -> ui256 attributes {id = {{[0-9]+}} : i64, state_mutability = #Pure}
// CHECK:   sol.call @{{.*bare.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK: sol.func @{{.*total.*}}
// CHECK:   sol.call @"operator_add({{[^"]*}}"
// CHECK: sol.func @"operator_add({{[^"]*}}"
// CHECK: sol.func @{{.*planted.*}}()
// CHECK: } {kind = #Contract}

type Score is uint256;

using {operator_add as +} for Score global;

function operator_add(Score a, Score b) pure returns (Score) {
    return Score.wrap(Score.unwrap(a) + Score.unwrap(b));
}

library Lib {
    function bare(uint256 a) internal pure returns (uint256) {
        return a * 3;
    }

    function calls_public_sibling(uint256 a) internal pure returns (uint256) {
        return public_sibling(a);
    }

    function public_sibling(uint256 a) public pure returns (uint256) {
        return bare(a);
    }

    function seed(uint256 a) internal pure returns (uint256) {
        return step(a);
    }

    function step(uint256 a) internal pure returns (uint256) {
        return a + 1;
    }
}

contract User {
    uint256 public planted = Lib.seed(21);

    function consume(uint256 x) public pure returns (uint256) {
        return Lib.calls_public_sibling(x);
    }

    function total(Score x) public pure returns (Score) {
        return operator_add(x, x) + x;
    }
}
