// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*f.*}}
// CHECK: sol.constant {{.*}} : ui32
// CHECK: sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>

contract C {
    struct S {
        bytes4 tag;
        uint256 n;
    }

    function f() public pure returns (bytes4) {
        S memory s = S("abcd", 7);
        return s.tag;
    }
}
