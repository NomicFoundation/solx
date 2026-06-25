// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A string literal stored into a single `byte` element of a `bytes` storage var
// (`data[0] = "x"`) materialises as a compile-time `!sol.byte` constant: the first
// literal byte becomes a ui8 constant (120 = 'x'), then `sol.bytes_cast ui8 to
// !sol.byte`, stored through the element pointer. Both backends emit the identical
// gep -> constant -> bytes_cast -> store body. (solc additionally emits an unused
// `sol.string_lit`; we pin only the shared byte-materialisation arm.)

// CHECK: sol.func @{{.*f.*}}()
// CHECK:   %[[BASE:.*]] = sol.addr_of @{{.*data.*}} : !sol.string<Storage>
// CHECK:   %[[I:.*]] = sol.constant 0 : ui8
// CHECK:   %[[P:.*]] = sol.gep %[[BASE]], %[[I]] : !sol.string<Storage>, ui8, !sol.ptr<!sol.byte, Storage>
// CHECK:   %[[C:.*]] = sol.constant 120 : ui8
// CHECK:   %[[B:.*]] = sol.bytes_cast %[[C]] : ui8 to !sol.byte
// CHECK:   sol.store %[[B]], %[[P]] : !sol.byte, !sol.ptr<!sol.byte, Storage>

contract C {
    bytes data;
    function f() public {
        data[0] = "x";
    }
}
