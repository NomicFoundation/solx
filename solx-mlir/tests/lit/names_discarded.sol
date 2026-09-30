// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*abiNamespace.*}}()
// CHECK-NEXT:   sol.return

// CHECK: sol.func @{{.*abiReference.*}}()
// CHECK-NEXT:   sol.return

// CHECK: sol.func @{{.*arrayReference.*}}()
// CHECK-NEXT:   sol.addr_of @{{.*data.*}}
// CHECK-NEXT:   sol.return

// CHECK: sol.func @{{.*arrayType.*}}()
// CHECK-NEXT:   sol.return

// CHECK: sol.func @{{.*elementaryType.*}}()
// CHECK-NEXT:   sol.return

// CHECK: sol.func @{{.*errors.*}}()
// CHECK-NEXT:   sol.return

// CHECK: sol.func @{{.*events.*}}()
// CHECK-NEXT:   sol.return

// CHECK: sol.func @{{.*operand.*}}()
// CHECK-NEXT:   sol.call @{{.*recipient.*}}()
// CHECK-NEXT:   sol.return

// CHECK: sol.func private @{{.*recipient.*}}()
// CHECK-NEXT:   %[[THIS:.*]] = sol.this
// CHECK-NEXT:   %[[SELF:.*]] = sol.address_cast %[[THIS]]
// CHECK-NEXT:   %[[PAYABLE:.*]] = sol.address_cast %[[SELF]]
// CHECK-NEXT:   sol.return %[[PAYABLE]]

// CHECK: sol.func @{{.*options.*}}()
// CHECK-NEXT:   sol.call @{{.*recipient.*}}()
// CHECK-NEXT:   sol.gasleft
// CHECK-NEXT:   sol.gasleft
// CHECK-NEXT:   sol.return

// CHECK: sol.func @{{.*receiverValue.*}}()
// CHECK-NEXT:   %[[RECIPIENT:.*]] = sol.call @{{.*recipient.*}}()
// CHECK-NEXT:   %[[ADDRESS:.*]] = sol.address_cast %[[RECIPIENT]]
// CHECK-NEXT:   sol.balance %[[ADDRESS]]
// CHECK-NEXT:   sol.return

// CHECK: sol.func @{{.*structs.*}}()
// CHECK-NEXT:   sol.return

// CHECK: sol.func @{{.*typeReference.*}}()
// CHECK-NEXT:   sol.return

struct Pair {
    uint256 first;
}

error Missing(uint256 code);

event Logged(uint256 code);

contract C {
    struct Inner {
        uint256 value;
    }

    error Absent(uint256 code);

    event Traced(uint256 code);

    uint256[] data;

    function recipient() internal view returns (address payable) {
        return payable(address(this));
    }

    function abiNamespace() public pure { abi; }

    function abiReference() public pure { abi.encode; }

    function arrayReference() public view { data.pop; }

    function arrayType() public pure { Pair[7][]; }

    function elementaryType() public pure { uint256; }

    function operand() public view { recipient().transfer; }

    function options() public view { recipient().call{value: gasleft(), gas: gasleft()}; }

    function receiverValue() public view { recipient().balance; }

    function structs() public pure { Pair; Inner; }

    function errors() public pure { Missing; Absent; }

    function events() public pure { Logged; Traced; }

    function typeReference() public pure { type(uint256); }
}
