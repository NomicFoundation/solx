// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Attached internal calls: solc's print-init drops the receiver from the argument list, so this
// is solx-only.

// CHECK: sol.func @{{.*plain.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.load
// CHECK:   %[[ARG:.*]] = sol.cast
// CHECK:   sol.call @{{.*bump.*}}(%[[RECEIVER]], %[[ARG]]) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*named.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.load
// CHECK:   sol.constant 4 : ui8
// CHECK:   sol.call @{{.*bump.*}}(%[[RECEIVER]], %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*storage_receiver.*}}
// CHECK:   %[[BOX:.*]] = sol.addr_of @{{.*stored.*}} : !sol.struct<(ui256), Storage>
// CHECK:   sol.call @{{.*fill.*}}(%[[BOX]], %{{.*}}) : (!sol.struct<(ui256), Storage>, ui256) -> ()

// CHECK: sol.func @{{.*free_function.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.load
// CHECK:   sol.call @{{.*double.*}}(%[[RECEIVER]]) : (ui256) -> ui256

function double(uint256 a) pure returns (uint256) {
    return a * 2;
}

library Lib {
    struct Box {
        uint256 value;
    }

    function bump(uint256 a, uint256 b) internal pure returns (uint256) {
        return a + b;
    }

    function fill(Box storage box, uint256 value) internal {
        box.value = value;
    }
}

contract User {
    using Lib for uint256;
    using Lib for Lib.Box;
    using {double} for uint256;

    Lib.Box stored;

    function plain(uint256 x) public pure returns (uint256) {
        return x.bump(1);
    }

    function named(uint256 x) public pure returns (uint256) {
        return x.bump({b: 4});
    }

    function storage_receiver(uint256 x) public {
        stored.fill(x);
    }

    function free_function(uint256 x) public pure returns (uint256) {
        return x.double();
    }
}
