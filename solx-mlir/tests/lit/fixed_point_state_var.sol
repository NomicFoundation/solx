// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A fixed-point state variable is a valid declaration; Type::resolve maps it to a
// signed or unsigned integer of the type's bit width. solc's MLIR backend does
// not emit fixed-point, so this is solx-only.

// CHECK: sol.state_var {{.*}}: si128
// CHECK: sol.state_var {{.*}}: ui128
// CHECK: sol.state_var {{.*}}: si64

contract C {
    fixed128x18 a;
    ufixed128x18 b;
    fixed64x10 c;
}
