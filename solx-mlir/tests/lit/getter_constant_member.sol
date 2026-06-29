// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Public-constant auto-getter whose initializer reads a library constant member: solc's print-init aborts NYI (UNREACHABLE at SolidityToMLIR.cpp:1698), so this is solx-only.

// CHECK-LABEL: sol.func @"DERIVED()"() -> ui256 attributes {{.*}}selector = 1646776813 : i32{{.*}}#Pure
// CHECK:   %[[C:.*]] = sol.constant 7 : ui8
// CHECK:   %[[V:.*]] = sol.cast %[[C]] : ui8 to ui256
// CHECK:   sol.return %[[V]] : ui256

library Lib {
    uint256 internal constant BASE = 7;
}

contract C {
    uint256 public constant DERIVED = Lib.BASE;
}
