// RUN: slang --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*f.*}}()
// CHECK:   %[[BASE:.*]] = sol.addr_of @{{.*data.*}} : !sol.string<Storage>
// CHECK:   %[[I:.*]] = sol.constant 0 : ui8
// CHECK:   %[[P:.*]] = sol.gep %[[BASE]], %[[I]] : !sol.string<Storage>, ui8, !sol.ptr<!sol.byte, Storage>
// CHECK:   %[[C:.*]] = sol.constant 120 : ui8
// CHECK:   %[[B:.*]] = sol.bytes_cast %[[C]] : ui8 to !sol.byte
// CHECK:   sol.store %[[B]], %[[P]] : !sol.byte, !sol.ptr<!sol.byte, Storage>

// CHECK: sol.func @{{.*parenthesized.*}}()
// CHECK:   %[[PAREN:.*]] = sol.constant 120 : ui8
// CHECK:   %[[PARENB:.*]] = sol.bytes_cast %[[PAREN]] : ui8 to !sol.byte
// CHECK:   sol.store %[[PARENB]], %{{.*}} : !sol.byte, !sol.ptr<!sol.byte, Storage>

// CHECK: sol.func @{{.*parenthesized_declaration.*}}()
// CHECK:   %[[DECL:.*]] = sol.constant 120 : ui8
// CHECK:   %[[DECLB:.*]] = sol.bytes_cast %[[DECL]] : ui8 to !sol.fixedbytes<1>
// CHECK:   sol.store %[[DECLB]], %{{.*}} : !sol.fixedbytes<1>, !sol.ptr<!sol.fixedbytes<1>, Stack>

contract C {
    bytes data;

    function f() public {
        data[0] = "x";
    }

    function parenthesized() public {
        data[0] = ("x");
    }

    function parenthesized_declaration() public pure returns (bytes1) {
        bytes1 b = ("x");
        return b;
    }
}
