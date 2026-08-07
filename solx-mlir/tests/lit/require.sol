// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*check.*}}
// CHECK:   %[[COND:.*]] = sol.cmp gt
// CHECK:   sol.require %[[COND]]()

// CHECK: sol.func @{{.*check_msg.*}}
// CHECK:   %[[COND:.*]] = sol.cmp gt
// CHECK:   sol.require %[[COND]], "must be positive"()

// CHECK: sol.func @{{.*check_error.*}}
// CHECK:   %[[COND:.*]] = sol.cmp gt
// CHECK:   sol.require %[[COND]], "MyError(uint256)"(%{{.*}}) {call}

// CHECK: sol.func @{{.*check_library_error.*}}
// CHECK:   %[[COND:.*]] = sol.cmp gt
// CHECK:   sol.require %[[COND]], "LibraryError(uint256)"(%{{.*}}) {call}

// CHECK: sol.func @{{.*check_named_arguments.*}}
// CHECK:   sol.require %{{.*}}, "MyErr(uint256,uint256)"(%{{.*}}, %{{.*}}) {call}

error MyError(uint256 x);

library Library {
    error LibraryError(uint256 x);
}

contract C {
    error MyErr(uint256 a, uint256 b);

    function check(uint256 x) public pure returns (uint256) {
        require(x > 0);
        return x;
    }

    function check_msg(uint256 x) public pure returns (uint256) {
        require(x > 0, "must be positive");
        return x;
    }

    function check_error(uint256 a) public pure {
        require(a > 0, MyError(a));
    }

    function check_library_error(uint256 a) public pure {
        require(a > 0, Library.LibraryError(a));
    }

    function check_named_arguments(bool ok) public pure {
        require(ok, MyErr({b: 99, a: 11}));
    }
}
