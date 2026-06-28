// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.state_var {{.*}}: si128
// CHECK: sol.state_var {{.*}}: ui128
// CHECK: sol.state_var {{.*}}: si64

contract C {
    fixed128x18 a;
    ufixed128x18 b;
    fixed64x10 c;
}
