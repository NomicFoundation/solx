// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solx defines a modifier right after the function that first names it, where solc's print-init
// lists a contract's modifiers after that contract's own functions, so this is solx-only.

// A `sol.modifier` belongs to the symbol table of every module that names it, so the library
// carries its own copy of the one the contracts also invoke.
// CHECK: sol.contract @{{.*}}Checks
// CHECK:   sol.modifier_invocation @[[LIBRARY_MODIFIER:.*]] {
// CHECK:   sol.modifier @[[LIBRARY_MODIFIER]]() {

// An interface base in the constructor's modifier list is a base-constructor entry without a
// constructor to call, so nothing comes between the base call and the modifier invocation.
// CHECK: sol.contract @{{.*}}Leaf
// CHECK: sol.func @{{.*}}(%arg0: ui256) attributes {kind = #{{.*}}Constructor
// CHECK:   sol.call @[[BASE_CONSTRUCTOR:.*]](%{{.*}}) : (ui256) -> ()
// CHECK-NEXT: sol.modifier_invocation @[[OVERRIDE:.*]] {
// CHECK:     sol.yield %{{.*}} : ui256
// CHECK: sol.func @[[BASE_CONSTRUCTOR]](%arg0: ui256) attributes {state_mutability
// CHECK:   sol.modifier_invocation @[[OVERRIDE]] {
// CHECK: sol.modifier @[[OVERRIDE]](%arg0: ui256) {
// CHECK:   sol.constant 100 : ui8
// CHECK: sol.func @{{.*}}qualified{{.*}}(%arg0: ui256) -> ui256
// CHECK:   sol.modifier_invocation @[[DECLARATION:.*]] {
// CHECK: sol.modifier @[[DECLARATION]](%arg0: ui256) {
// CHECK:   sol.constant 10 : ui8
// CHECK: sol.func @{{.*}}attached{{.*}}(%arg0: ui256) -> ui256
// CHECK:   sol.call @[[LIBRARY_FUNCTION:.*]](%{{.*}}) : (ui256) -> ui256
// CHECK: sol.func @[[LIBRARY_FUNCTION]](%arg0: ui256) -> ui256
// CHECK:   sol.modifier_invocation @[[LIBRARY_MODIFIER]] {
// CHECK: sol.modifier @[[LIBRARY_MODIFIER]]() {
// CHECK:   sol.constant 1000 : ui16
// CHECK: sol.func @{{.*}}() attributes {kind = #{{.*}}Receive
// CHECK:   sol.modifier_invocation @[[IMPLEMENTED:.*]] {
// CHECK: sol.modifier @[[IMPLEMENTED]]() {
// CHECK:   sol.load_immutable @{{.*}}limit{{.*}} : ui256
// CHECK:   sol.placeholder
// CHECK-NEXT: sol.return
// CHECK-NEXT: }
// CHECK: sol.func @{{.*}}inherited{{.*}}(%arg0: ui256) -> ui256
// CHECK:   sol.modifier_invocation @[[OVERRIDE]] {

library Checks {
    modifier bounded() {
        require(gasleft() < 1000);
        _;
    }

    function clamp(uint256 x) internal view bounded returns (uint256) {
        return x;
    }
}

abstract contract Base {
    uint256 immutable limit = 1;

    modifier bounded(uint256 x) virtual {
        require(x < 10);
        _;
    }

    modifier funded() virtual;

    constructor(uint256 x) bounded(x) {}

    function inherited(uint256 x) public bounded(x) returns (uint256) {
        return x;
    }
}

interface Marker {}

contract Leaf is Base, Marker {
    modifier bounded(uint256 x) override {
        require(x < 100);
        _;
    }

    modifier funded() override {
        require(msg.value >= limit);
        _;
        return;
    }

    constructor(uint256 x) Base(x) Marker() bounded(x) {}

    function qualified(uint256 x) public Base.bounded(x) returns (uint256) {
        return x;
    }

    function attached(uint256 x) public view returns (uint256) {
        return Checks.clamp(x);
    }

    receive() external payable funded {}
}
