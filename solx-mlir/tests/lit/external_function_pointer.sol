// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc print-init evaluates a pointer call's arguments and options before its callee, unlike
// legacy, so this is solx-only.

// CHECK: sol.func @{{.*from_instance.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.address_cast %{{.*}} : !sol.contract<{{.*I.*}}> to !sol.address
// CHECK:   sol.ext_func_constant %[[RECEIVER]] {selector = -1277270901 : i32} : !sol.address -> !sol.ext_func_ref<(ui256) -> ui256>

// CHECK: sol.func @{{.*from_this.*}}
// CHECK:   %[[SELF:.*]] = sol.this : !sol.contract<{{.*C.*}}>
// CHECK:   %[[SELF_ADDRESS:.*]] = sol.address_cast %[[SELF]] : !sol.contract<{{.*C.*}}> to !sol.address
// CHECK:   sol.ext_func_constant %[[SELF_ADDRESS]] {selector = -1743665215 : i32} : !sol.address -> !sol.ext_func_ref<(ui256) -> ui256>

// CHECK: sol.func @{{.*selector_of.*}}
// CHECK:   %[[POINTER:.*]] = sol.load %{{.*}} : !sol.ptr<!sol.ext_func_ref<(ui256) -> ui256>, Stack>, !sol.ext_func_ref<(ui256) -> ui256>
// CHECK:   sol.ext_func_selector %[[POINTER]] : !sol.ext_func_ref<(ui256) -> ui256> -> !sol.fixedbytes<4>

// CHECK: sol.func @{{.*selector_of_named.*}}
// CHECK:   %[[SELECTOR:.*]] = sol.constant 2551302081 : ui32
// CHECK:   sol.bytes_cast %[[SELECTOR]] : ui32 to !sol.fixedbytes<4>

// CHECK: sol.func @{{.*address_of.*}}
// CHECK:   %[[POINTER:.*]] = sol.load %{{.*}} : !sol.ptr<!sol.ext_func_ref<(ui256) -> ui256>, Stack>, !sol.ext_func_ref<(ui256) -> ui256>
// CHECK:   sol.ext_func_addr %[[POINTER]] : !sol.ext_func_ref<(ui256) -> ui256> -> !sol.address

// CHECK: sol.func @{{.*address_of_named.*}}
// CHECK:   %[[NAMED:.*]] = sol.ext_func_constant %{{.*}} {selector = -1743665215 : i32} : !sol.address -> !sol.ext_func_ref<(ui256) -> ui256>
// CHECK:   sol.ext_func_addr %[[NAMED]] : !sol.ext_func_ref<(ui256) -> ui256> -> !sol.address

// CHECK: sol.func @{{.*call.*}}
// CHECK:   %[[POINTER:.*]] = sol.load %{{.*}} : !sol.ptr<!sol.ext_func_ref<(ui256) -> ui256>, Stack>, !sol.ext_func_ref<(ui256) -> ui256>
// CHECK:   %[[ARGUMENT:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   %[[LEFT:.*]] = sol.gasleft : ui256
// CHECK:   %[[ZERO:.*]] = sol.constant 0 : ui256
// CHECK:   sol.ext_icall %[[POINTER]](%[[ARGUMENT]]) gas %[[LEFT]] value %[[ZERO]] : !sol.ext_func_ref<(ui256) -> ui256>, (ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*call_view.*}}
// CHECK:   %[[POINTER:.*]] = sol.load %{{.*}} : !sol.ptr<!sol.ext_func_ref<(ui256) -> ui256>, Stack>, !sol.ext_func_ref<(ui256) -> ui256>
// CHECK:   %[[ARGUMENT:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.ext_icall %[[POINTER]](%[[ARGUMENT]]) gas %{{.*}} value %{{.*}} {static_call} : !sol.ext_func_ref<(ui256) -> ui256>, (ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*call_storage.*}}
// CHECK:   %[[POINTER:.*]] = sol.load %{{.*}} : !sol.ptr<!sol.ext_func_ref<(!sol.array<? x ui256, Memory>) -> ui256>, Stack>, !sol.ext_func_ref<(!sol.array<? x ui256, Memory>) -> ui256>
// CHECK:   %[[SLOT:.*]] = sol.addr_of @{{.*}} : !sol.array<? x ui256, Storage>
// CHECK:   sol.ext_icall %[[POINTER]](%[[SLOT]]) gas %{{.*}} value %{{.*}} : !sol.ext_func_ref<(!sol.array<? x ui256, Memory>) -> ui256>, (!sol.array<? x ui256, Storage>) -> (i1, ui256)

// CHECK: sol.func @{{.*call_options.*}}
// CHECK:   %[[POINTER:.*]] = sol.load %{{.*}} : !sol.ptr<!sol.ext_func_ref<(ui256) -> ui256>, Stack>, !sol.ext_func_ref<(ui256) -> ui256>
// CHECK:   %[[VALUE:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   %[[GAS:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   %[[ARGUMENT:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.ext_icall %[[POINTER]](%[[ARGUMENT]]) gas %[[GAS]] value %[[VALUE]] : !sol.ext_func_ref<(ui256) -> ui256>, (ui256) -> (i1, ui256)

contract C {
    uint256[] stored;

    function target(uint256 a) external returns (uint256) {
        return a;
    }

    function from_instance(I i) public pure returns (function(uint256) external returns (uint256)) {
        return i.f;
    }

    function from_this() public view returns (function(uint256) external returns (uint256)) {
        return this.target;
    }

    function selector_of(function(uint256) external returns (uint256) p) public pure returns (bytes4) {
        return p.selector;
    }

    function selector_of_named() public pure returns (bytes4) {
        return this.target.selector;
    }

    function address_of(function(uint256) external returns (uint256) p) public pure returns (address) {
        return p.address;
    }

    function address_of_named() public view returns (address) {
        return this.target.address;
    }

    function call(function(uint256) external returns (uint256) p, uint256 x) public returns (uint256) {
        return p(x);
    }

    function call_view(function(uint256) external view returns (uint256) p, uint256 x) public view returns (uint256) {
        return p(x);
    }

    function call_storage(function(uint256[] memory) external returns (uint256) p) public returns (uint256) {
        return p(stored);
    }

    function call_options(function(uint256) external payable returns (uint256) p, uint256 v, uint256 g) public returns (uint256) {
        return p{value: v, gas: g}(7);
    }
}

interface I {
    function f(uint256 a) external returns (uint256);
}
