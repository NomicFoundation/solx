// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc print-init evaluates a call's options before its receiver, unlike legacy, so this is
// solx-only.

// CHECK: sol.func @{{.*plain.*}}
// CHECK:   sol.address_cast %{{.*}} : !sol.contract<{{.*I.*}}> to !sol.address
// CHECK:   %[[SELECTOR:.*]] = sol.constant 3017696395 : ui256
// CHECK:   sol.gasleft
// CHECK:   sol.ext_call "{{.*f.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %[[SELECTOR]] {callee_type = (ui256) -> ui256} : !sol.address, (ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*view_callee.*}}
// CHECK:   %[[SELECTOR:.*]] = sol.constant 3827312202 : ui256
// CHECK:   sol.ext_call "{{.*g.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %[[SELECTOR]] {callee_type = (ui256) -> ui256, static_call} : !sol.address, (ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*named.*}}
// CHECK:   sol.constant 11 : ui8
// CHECK:   sol.constant 99 : ui8
// CHECK:   sol.ext_call "{{.*h.*}}"(%{{.*}}, %{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256, ui256) -> ui256} : !sol.address, (ui256, ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*storage_argument.*}}
// CHECK:   %[[SLOT:.*]] = sol.addr_of @{{.*}} : !sol.array<? x ui256, Storage>
// CHECK:   sol.ext_call "{{.*r.*}}"(%[[SLOT]]) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (!sol.array<? x ui256, Memory>) -> ui256} : !sol.address, (!sol.array<? x ui256, Storage>) -> (i1, ui256)

// CHECK: sol.func @{{.*value_then_gas.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.address_cast %{{.*}} : !sol.contract<{{.*I.*}}> to !sol.address
// CHECK:   %[[V:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   %[[G:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.ext_call "{{.*p.*}}"(%{{.*}}) at %[[RECEIVER]] gas %[[G]] value %[[V]] selector

// CHECK: sol.func @{{.*gas_then_value.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.address_cast %{{.*}} : !sol.contract<{{.*I.*}}> to !sol.address
// CHECK:   %[[G:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   %[[V:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.ext_call "{{.*p.*}}"(%{{.*}}) at %[[RECEIVER]] gas %[[G]] value %[[V]] selector

// CHECK: sol.func @{{.*gas_only.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.address_cast %{{.*}} : !sol.contract<{{.*I.*}}> to !sol.address
// CHECK:   %[[G:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   %[[ZERO:.*]] = sol.constant 0 : ui256
// CHECK:   sol.ext_call "{{.*p.*}}"(%{{.*}}) at %[[RECEIVER]] gas %[[G]] value %[[ZERO]] selector

// CHECK: sol.func @{{.*value_only.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.address_cast %{{.*}} : !sol.contract<{{.*I.*}}> to !sol.address
// CHECK:   %[[V:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   %[[LEFT:.*]] = sol.gasleft
// CHECK:   sol.ext_call "{{.*p.*}}"(%{{.*}}) at %[[RECEIVER]] gas %[[LEFT]] value %[[V]] selector

// CHECK: sol.func @{{.*parenthesized.*}}
// CHECK:   %[[V:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.ext_call "{{.*p.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %[[V]] selector

// CHECK: sol.func @{{.*parenthesized_reference.*}}
// CHECK:   sol.ext_call "{{.*g.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256, static_call}

contract C {
    uint256[] stored;

    function plain(I i) public returns (uint256) {
        return i.f(1);
    }

    function view_callee(I i) public view returns (uint256) {
        return i.g(2);
    }

    function named(I i) public returns (uint256) {
        return i.h({b: 99, a: 11});
    }

    function storage_argument(I i) public returns (uint256) {
        return i.r(stored);
    }

    function value_then_gas(I i, uint256 v, uint256 g) public returns (uint256) {
        return i.p{value: v, gas: g}(3);
    }

    function gas_then_value(I i, uint256 v, uint256 g) public returns (uint256) {
        return i.p{gas: g, value: v}(4);
    }

    function gas_only(I i, uint256 g) public returns (uint256) {
        return i.p{gas: g}(5);
    }

    function value_only(I i, uint256 v) public returns (uint256) {
        return i.p{value: v}(6);
    }

    function parenthesized(I i, uint256 v) public returns (uint256) {
        return (i.p{value: v})(7);
    }

    function parenthesized_reference(I i, uint256 x) public view returns (uint256) {
        return (i.g)(x);
    }
}

interface I {
    function f(uint256 a) external returns (uint256);

    function g(uint256 a) external view returns (uint256);

    function h(uint256 a, uint256 b) external returns (uint256);

    function r(uint256[] calldata a) external returns (uint256);

    function p(uint256 a) external payable returns (uint256);
}
