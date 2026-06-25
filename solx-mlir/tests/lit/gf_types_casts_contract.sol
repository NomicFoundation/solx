// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// contract <-> address conversions route through `sol.address_cast`; a
// contract-to-contract conversion (here an upcast Derived -> Base) routes
// through `sol.contract_cast`. solc suffixes contract names with an id
// (Other_1, Base_1), so the contract type names are matched by regex. Both
// backends agree on the ops; function order differs so CHECK-DAG is used.

// CHECK-DAG: sol.func @{{.*to_contract.*}}(%{{.*}}: !sol.address) -> !sol.contract<{{.*Other.*}}>
// CHECK-DAG:   sol.address_cast %{{.*}} : !sol.address to !sol.contract<{{.*Other.*}}>

// CHECK-DAG: sol.func @{{.*from_contract.*}}(%{{.*}}: !sol.contract<{{.*Other.*}}>) -> !sol.address
// CHECK-DAG:   sol.address_cast %{{.*}} : !sol.contract<{{.*Other.*}}> to !sol.address

// CHECK-DAG: sol.func @{{.*upcast.*}}(%{{.*}}: !sol.contract<{{.*Derived.*}}>) -> !sol.contract<{{.*Base.*}}>
// CHECK-DAG:   sol.contract_cast %{{.*}} : <{{.*Derived.*}}> to <{{.*Base.*}}>

contract Other {}
contract Base {}
contract Derived is Base {}

contract C {
    function to_contract(address a) public pure returns (Other) { return Other(a); }
    function from_contract(Other o) public pure returns (address) { return address(o); }
    function upcast(Derived d) public pure returns (Base) { return Base(d); }
}
