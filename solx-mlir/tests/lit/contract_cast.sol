// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// address <-> contract / interface conversions route through sol.address_cast in
// both directions; a contract-to-contract conversion (here the upcast Derived to
// Base) routes through sol.contract_cast. Contract symbols carry a node-id suffix,
// matched by regex.

// CHECK-DAG: sol.func @{{.*to_contract.*}}
// CHECK-DAG:   sol.address_cast %{{.*}} : !sol.address to !sol.contract<{{.*Other.*}}>

// CHECK-DAG: sol.func @{{.*from_contract.*}}
// CHECK-DAG:   sol.address_cast %{{.*}} : !sol.contract<{{.*Other.*}}> to !sol.address

// CHECK-DAG: sol.func @{{.*to_interface.*}}
// CHECK-DAG:   sol.address_cast %{{.*}} : !sol.address to !sol.contract<{{.*I.*}}>

// CHECK-DAG: sol.func @{{.*from_interface.*}}
// CHECK-DAG:   sol.address_cast %{{.*}} : !sol.contract<{{.*I.*}}> to !sol.address

// CHECK-DAG: sol.func @{{.*upcast.*}}
// CHECK-DAG:   sol.contract_cast %{{.*}} : <{{.*Derived.*}}> to <{{.*Base.*}}>

interface I {}

contract Other {}

contract Base {}

contract Derived is Base {}

contract C {
    function to_contract(address a) public pure returns (Other) {
        return Other(a);
    }

    function from_contract(Other o) public pure returns (address) {
        return address(o);
    }

    function to_interface(address a) public pure returns (I) {
        return I(a);
    }

    function from_interface(I i) public pure returns (address) {
        return address(i);
    }

    function upcast(Derived d) public pure returns (Base) {
        return Base(d);
    }
}
