// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.contract @{{.*}}Checks
// CHECK:   sol.modifier_invocation @[[LIBRARY_MODIFIER:.*]] {
// CHECK:   sol.modifier @[[LIBRARY_MODIFIER]]() {

// CHECK: sol.contract @{{.*}}Leaf
// CHECK: sol.func @{{.*}}(%arg0: ui256) attributes {kind = #{{.*}}Constructor
// CHECK:   sol.store %arg0, %[[X:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.call @[[BASE_CONSTRUCTOR:.*]](%{{.*}}) : (ui256) -> ()
// CHECK:   sol.modifier_invocation @[[OVERRIDE:.*]] {
// CHECK:     %[[ARGUMENT:.*]] = sol.load %[[X]]
// CHECK:     sol.yield %[[ARGUMENT]] : ui256

// CHECK: sol.func @[[BASE_CONSTRUCTOR]](%arg0: ui256) attributes {state_mutability
// CHECK:   sol.modifier_invocation @[[OVERRIDE]] {

// CHECK: sol.modifier @[[OVERRIDE]](%arg0: ui256) {
// CHECK:   sol.store %arg0, %[[LIMIT:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.constant 100 : ui8
// CHECK:   %[[VALUE:.*]] = sol.load %[[LIMIT]]
// CHECK:   sol.cmp lt, %[[VALUE]]
// CHECK:   sol.require
// CHECK:   sol.placeholder

// CHECK: sol.func @{{.*}}qualified{{.*}}(%arg0: ui256) -> ui256
// CHECK:   sol.modifier_invocation @[[DECLARATION:.*]] {

// CHECK: sol.modifier @[[DECLARATION]](%arg0: ui256) {
// CHECK:   sol.constant 10 : ui8
// CHECK:   sol.placeholder

// CHECK: sol.func @{{.*}}attached{{.*}}(%arg0: ui256) -> ui256
// CHECK:   sol.call @[[LIBRARY_FUNCTION:.*]](%{{.*}}) : (ui256) -> ui256

// CHECK: sol.func @[[LIBRARY_FUNCTION]](%arg0: ui256) -> ui256
// CHECK:   sol.modifier_invocation @[[LIBRARY_MODIFIER]] {

// CHECK: sol.modifier @[[LIBRARY_MODIFIER]]() {
// CHECK:   sol.constant 1000 : ui16
// CHECK:   sol.gasleft
// CHECK:   sol.placeholder

// CHECK: sol.func @{{.*}}() attributes {kind = #{{.*}}Receive
// CHECK:   sol.modifier_invocation @[[FUNDED:.*]] {

// CHECK: sol.modifier @[[FUNDED]]() {
// CHECK:   sol.load_immutable
// CHECK:   sol.callvalue
// CHECK:   sol.placeholder

// CHECK: sol.func @{{.*}}inherited{{.*}}(%arg0: ui256) -> ui256
// CHECK:   sol.modifier_invocation @[[OVERRIDE]] {

library Checks {
    modifier bounded() {
        require(gasleft() < 1000);
        _;
    }

    function clamp(uint256 x) internal view bounded returns (uint256) {}
}

abstract contract Base {
    uint256 immutable limit = 1;

    modifier bounded(uint256 x) virtual {
        require(x < 10);
        _;
    }

    constructor(uint256 x) bounded(x) {}

    function inherited(uint256 x) public bounded(x) returns (uint256) {}
}

contract Leaf is Base {
    modifier bounded(uint256 x) override {
        require(x < 100);
        _;
    }

    modifier funded() {
        require(msg.value >= limit);
        _;
    }

    constructor(uint256 x) Base(x) bounded(x) {}

    function qualified(uint256 x) public Base.bounded(x) returns (uint256) {}

    function attached(uint256 x) public view returns (uint256) {
        return Checks.clamp(x);
    }

    receive() external payable funded {}
}
