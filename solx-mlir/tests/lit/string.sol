// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func {{.*}}bytes_id{{.*}}!sol.string<Memory>{{.*}}!sol.string<Memory>

// CHECK: sol.func {{.*}}string_id{{.*}}!sol.string<Memory>{{.*}}!sol.string<Memory>

contract C {
    function string_id(string memory s) public pure returns (string memory) { return s; }

    function bytes_id(bytes memory b) public pure returns (bytes memory) { return b; }
}
