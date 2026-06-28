// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// `sol.bytes_cast` only connects a fixed-bytes type with its matching-width
// integer (`fixedbytes<N>` <-> `ui(N*8)`, `byte` <-> `ui8`), so comparing a
// `bytes` element against `0x05` needs a partner integer to bridge widths.
// solx casts the loaded `fixedbytes<1>` to `ui8`, widens both sides to `ui256`,
// and emits `sol.cmp eq ... : ui256`; solc casts the literal up to
// `fixedbytes<1>` and emits `sol.cmp eq ... : !sol.fixedbytes<1>`. CHECK-SOLX
// pins the integer path, CHECK-SOLC the fixed-bytes path.

// CHECK: sol.load %{{.*}} : !sol.ptr<!sol.byte, Storage>, !sol.byte
// CHECK: sol.bytes_cast %{{.*}} : !sol.byte to !sol.fixedbytes<1>
// CHECK-SOLX: sol.bytes_cast %{{.*}} : !sol.fixedbytes<1> to ui8
// CHECK-SOLX: sol.cmp eq, %{{.*}}, %{{.*}} : ui256
// CHECK-SOLC: sol.bytes_cast %{{.*}} : ui8 to !sol.fixedbytes<1>
// CHECK-SOLC: sol.cmp eq, %{{.*}}, %{{.*}} : !sol.fixedbytes<1>

contract C {
    bytes data;

    function f() public returns (bool) {
        data.push(0x05);
        return data[0] == 0x05;
    }
}
