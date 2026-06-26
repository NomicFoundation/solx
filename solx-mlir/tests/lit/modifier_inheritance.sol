// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// CHECK-SOLX: sol.contract @Derived
// CHECK-SOLX: sol.func @"inherited(uint256)"(%arg0: ui256) -> ui256
// CHECK-SOLX: sol.call @[[DERIVED:checked_[0-9]+]](%{{.*}}) : (ui256) -> ()
// CHECK-SOLX: sol.func @"qualified(uint256)"(%arg0: ui256) -> ui256
// CHECK-SOLX: sol.call @[[BASE:checked_[0-9]+]](%{{.*}}) : (ui256) -> ()
// CHECK-SOLX: sol.modifier @[[DERIVED]](%arg0: ui256)
// CHECK-SOLX: sol.cmp ne
// CHECK-SOLX: sol.modifier @[[BASE]](%arg0: ui256)
// CHECK-SOLX: sol.cmp gt

// CHECK-SOLC: sol.contract @Derived
// CHECK-SOLC: sol.func @qualified
// CHECK-SOLC: sol.call @[[BASE:checked_[0-9]+]](%{{.*}}) : (ui256) -> ()
// CHECK-SOLC: sol.modifier @[[DERIVED:checked_[0-9]+]](%arg0: ui256)
// CHECK-SOLC: sol.cmp ne
// CHECK-SOLC: sol.func @inherited
// CHECK-SOLC: sol.call @[[BASE]](%{{.*}}) : (ui256) -> ()
// CHECK-SOLC: sol.modifier @[[BASE]](%arg0: ui256)
// CHECK-SOLC: sol.cmp gt

contract Base {
    modifier checked(uint256 v) virtual {
        require(v > 0);
        _;
    }

    function inherited(uint256 v) public checked(v) returns (uint256) {
        return v;
    }
}

contract Derived is Base {
    modifier checked(uint256 v) override {
        require(v != 1);
        _;
    }

    function qualified(uint256 v) public Base.checked(v) returns (uint256) {
        return v + 1;
    }
}
