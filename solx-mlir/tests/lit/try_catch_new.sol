// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*guarded.*}}
// CHECK:   %[[NEW:.*]] = sol.new "{{[^"]*}}Child{{[^"]*}}" value = %{{.*}} ctor(%{{.*}} : ui256) try : !sol.contract<"{{[^"]*}}Child{{[^"]*}}">
// CHECK:   %[[ADDRESS:.*]] = sol.address_cast %[[NEW]] : !sol.contract<"{{[^"]*}}Child{{[^"]*}}"> to !sol.address
// CHECK:   %[[RAW:.*]] = sol.address_cast %[[ADDRESS]] : !sol.address to ui160
// CHECK:   %[[ZERO:.*]] = sol.constant 0 : ui160
// CHECK:   %[[STATUS:.*]] = sol.cmp ne, %[[RAW]], %[[ZERO]] : ui160
// CHECK:   sol.try %[[STATUS]] {
// CHECK:   } panic {
// CHECK-NEXT: } error {
// CHECK-NEXT: ^bb{{[0-9]+}}(%{{.*}}: !sol.string<Memory>):
// CHECK:   } fallback {
// CHECK-NEXT:   sol.constant 1 : ui8

// CHECK: sol.func @{{.*value_and_salt.*}}
// CHECK:   sol.new "{{[^"]*}}Child{{[^"]*}}" value = %{{.*}} salt = %{{.*}} ctor(%{{.*}} : ui256) try : !sol.contract<"{{[^"]*}}Child{{[^"]*}}">

// CHECK: sol.func @{{.*named.*}}
// CHECK:   %[[FIRST:.*]] = sol.cast %c3_ui8
// CHECK:   %[[SECOND:.*]] = sol.cast %c4_ui8
// CHECK:   sol.new "{{[^"]*}}Pair{{[^"]*}}" value = %{{.*}} ctor(%[[FIRST]], %[[SECOND]] : ui256, ui256) try : !sol.contract<"{{[^"]*}}Pair{{[^"]*}}">

contract C {
    function guarded() public returns (address) {
        try new Child(1) returns (Child child) {
            return address(child);
        } catch Error(string memory reason) {
            return address(bytes20(bytes(reason)));
        } catch {
            return address(1);
        }
    }

    function value_and_salt(uint256 v, bytes32 s) public returns (address) {
        try new Child{value: v, salt: s}(2) returns (Child child) {
            return address(child);
        } catch {
            return address(0);
        }
    }

    function named() public returns (address) {
        try new Pair({b: 4, a: 3}) returns (Pair pair) {
            return address(pair);
        } catch {
            return address(0);
        }
    }
}

contract Pair {
    uint256 sum;

    constructor(uint256 a, uint256 b) {
        sum = a - b;
    }
}

contract Child {
    uint256 stored;

    constructor(uint256 a) payable {
        stored = a;
    }
}
