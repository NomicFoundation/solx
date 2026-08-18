// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*identity.*}}
// CHECK:   %[[VALUE:.*]] = sol.load
// CHECK-NEXT:   sol.return %[[VALUE]]

// CHECK: sol.func @{{.*address_to_u160.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.address to ui160

// CHECK: sol.func @{{.*address_to_u256.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.address to ui160
// CHECK:   sol.cast %{{.*}} : ui160 to ui256

// CHECK: sol.func @{{.*address_to_bytes20.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.address to !sol.fixedbytes<20>

// CHECK: sol.func @{{.*to_address.*}}
// CHECK:   sol.cast %{{.*}} : ui256 to ui160
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address

// CHECK: sol.func @{{.*u160_to_address.*}}
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address

// CHECK: sol.func @{{.*literal_to_address.*}}
// CHECK:   sol.constant 0 : ui8
// CHECK:   sol.cast %{{.*}} : ui8 to ui160
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address

// CHECK: sol.func @{{.*contract_to_address.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.contract<{{.*Other.*}}> to !sol.address

// CHECK: sol.func @{{.*this_to_address.*}}
// CHECK:   %[[SELF:.*]] = sol.this : !sol.contract<{{.*C.*}}>
// CHECK:   sol.address_cast %[[SELF]] : !sol.contract<{{.*C.*}}> to !sol.address

// CHECK: sol.func @{{.*interface_to_address.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.contract<{{.*I.*}}> to !sol.address

// CHECK: sol.func @{{.*address_to_contract.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.address to !sol.contract<{{.*Other.*}}>

// CHECK: sol.func @{{.*address_to_interface.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.address to !sol.contract<{{.*I.*}}>

contract C {
    function identity(address a) public pure returns (address) {
        return address(a);
    }

    function address_to_u160(address a) public pure returns (uint160) {
        return uint160(a);
    }

    function address_to_u256(address a) public pure returns (uint256) {
        return uint256(uint160(a));
    }

    function address_to_bytes20(address a) public pure returns (bytes20) {
        return bytes20(a);
    }

    function to_address(uint256 x) public pure returns (address) {
        return address(uint160(x));
    }

    function u160_to_address(uint160 u) public pure returns (address) {
        return address(u);
    }

    function literal_to_address() public pure returns (address) {
        return address(0);
    }

    function contract_to_address(Other o) public pure returns (address) {
        return address(o);
    }

    function this_to_address() public view returns (address) {
        return address(this);
    }

    function interface_to_address(I i) public pure returns (address) {
        return address(i);
    }

    function address_to_contract(address a) public pure returns (Other) {
        return Other(a);
    }

    function address_to_interface(address a) public pure returns (I) {
        return I(a);
    }
}

interface I {}

contract Other {}
