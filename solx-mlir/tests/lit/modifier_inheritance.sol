// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// inherited()'s virtual checked: solx binds the Derived override (cmp ne); solc print-init binds the Base modifier (cmp gt), same one qualified()'s Base.checked uses.
// solx emits functions then modifiers; solc interleaves function, modifier.

// CHECK: sol.contract @Derived

// CHECK-SOLX: sol.func @"inherited(uint256)"
// CHECK-SOLX: sol.call @[[INH:checked_[0-9]+]](%{{.*}}) : (ui256) -> ()
// CHECK-SOLX: sol.func @"qualified(uint256)"
// CHECK-SOLX: sol.call @[[QUAL:checked_[0-9]+]](%{{.*}}) : (ui256) -> ()
// CHECK-SOLX: sol.modifier @[[INH]](%arg0: ui256)
// CHECK-SOLX: sol.cmp ne
// CHECK-SOLX: sol.modifier @[[QUAL]](%arg0: ui256)
// CHECK-SOLX: sol.cmp gt

// CHECK-SOLC: sol.func @qualified
// CHECK-SOLC: sol.call @[[QUAL:checked_[0-9]+]](%{{.*}}) : (ui256) -> ()
// CHECK-SOLC: sol.modifier @[[OVR:checked_[0-9]+]](%arg0: ui256)
// CHECK-SOLC: sol.cmp ne
// CHECK-SOLC: sol.func @inherited
// CHECK-SOLC: sol.call @[[QUAL]](%{{.*}}) : (ui256) -> ()
// CHECK-SOLC: sol.modifier @[[QUAL]](%arg0: ui256)
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
