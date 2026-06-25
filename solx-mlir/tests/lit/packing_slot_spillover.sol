// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Exercises slot spillover during packing. address(20 bytes) + uint96(12 bytes)
// exactly fill slot 0; the full-width uint256 then takes slot 1 by itself; and
// the trailing bool + uint8 pack together into slot 2. Both backends agree on
// every slot/offset. Reads are a uniform addr_of -> load regardless of where
// the var sits in its slot, since the offset lives in the state_var decl.

// CHECK-DAG: sol.state_var @{{.*owner.*}} slot 0 offset 0 : !sol.address
// CHECK-DAG: sol.state_var @{{.*balance.*}} slot 0 offset 20 : ui96
// CHECK-DAG: sol.state_var @{{.*total.*}} slot 1 offset 0 : ui256
// CHECK-DAG: sol.state_var @{{.*flag.*}} slot 2 offset 0 : i1
// CHECK-DAG: sol.state_var @{{.*small.*}} slot 2 offset 1 : ui8

// CHECK: sol.func @{{.*getBalance.*}}() -> ui96
// CHECK:   %[[P:.*]] = sol.addr_of @{{.*balance.*}} : !sol.ptr<ui96, Storage>
// CHECK:   sol.load %[[P]] : !sol.ptr<ui96, Storage>, ui96

// CHECK: sol.func @{{.*getSmall.*}}() -> ui8
// CHECK:   %[[P:.*]] = sol.addr_of @{{.*small.*}} : !sol.ptr<ui8, Storage>
// CHECK:   sol.load %[[P]] : !sol.ptr<ui8, Storage>, ui8

contract C {
    address owner;
    uint96 balance;
    uint256 total;
    bool flag;
    uint8 small;

    function getBalance() public view returns (uint96) { return balance; }
    function getSmall() public view returns (uint8) { return small; }
    function getOwner() public view returns (address) { return owner; }
    function getTotal() public view returns (uint256) { return total; }
    function getFlag() public view returns (bool) { return flag; }
}
