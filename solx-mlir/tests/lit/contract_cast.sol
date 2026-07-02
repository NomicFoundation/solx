// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*from_contract.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.contract<{{.*Other.*}}> to !sol.address

// CHECK: sol.func @{{.*from_interface.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.contract<{{.*I.*}}> to !sol.address

// CHECK: sol.func @{{.*to_contract.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.address to !sol.contract<{{.*Other.*}}>

// CHECK: sol.func @{{.*to_interface.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.address to !sol.contract<{{.*I.*}}>

// CHECK: sol.func @{{.*upcast.*}}
// CHECK:   sol.contract_cast %{{.*}} : <{{.*Derived.*}}> to <{{.*Base.*}}>

interface I {}

contract Other {}

contract Base {}

contract Derived is Base {}

contract C {
    function from_contract(Other o) public pure returns (address) {
        return address(o);
    }

    function from_interface(I i) public pure returns (address) {
        return address(i);
    }

    function to_contract(address a) public pure returns (Other) {
        return Other(a);
    }

    function to_interface(address a) public pure returns (I) {
        return I(a);
    }

    function upcast(Derived d) public pure returns (Base) {
        return Base(d);
    }
}
