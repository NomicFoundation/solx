// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A cycle closed by a function type. solc does not follow function types when it marks a struct
// recursive, so its print-init recurses into the literal form until the stack overflows and emits
// nothing, while legacy compiles the file; this is solx-only.

// CHECK: sol.contract @{{.*}}C
// CHECK:   sol.state_var @{{.*}}closure{{.*}} slot 0 offset 0 : !sol.struct<"Closure_{{[0-9]+}}", Storage, (!sol.func_ref<(!sol.struct<"Closure_{{[0-9]+}}", Memory, (!sol.func_ref<(!sol.struct<"Closure_{{[0-9]+}}", Memory>) -> ui256>)>) -> ui256>)>
// CHECK:   sol.state_var @{{.*}}first{{.*}} slot 1 offset 0 : !sol.struct<"ArrayIntoFunctionCycle_{{[0-9]+}}", Storage, (!sol.array<? x !sol.struct<"FunctionBackToArray_{{[0-9]+}}", Storage, (!sol.func_ref<(!sol.struct<"ArrayIntoFunctionCycle_{{[0-9]+}}", Memory, {{.*}}) -> ()>)>, Storage>)>
// CHECK:   sol.state_var @{{.*}}second{{.*}} slot 2 offset 0 : !sol.struct<"FunctionBackToArray_{{[0-9]+}}", Storage, (!sol.func_ref<(!sol.struct<"ArrayIntoFunctionCycle_{{[0-9]+}}", Memory, {{.*}}) -> ()>)>
// CHECK:   sol.state_var @{{.*}}third{{.*}} slot 3 offset 0 : !sol.struct<(!sol.struct<"ArrayIntoFunctionCycle_{{[0-9]+}}", Storage, {{.*}}>), Storage>
// CHECK:   sol.state_var @{{.*}}multi{{.*}} slot 4 offset 0 : !sol.struct<"Multi_{{[0-9]+}}", Storage, (!sol.func_ref<() -> (!sol.struct<"Multi_{{[0-9]+}}", Memory, {{.*}}>, ui256)>)>
// CHECK:   sol.state_var @{{.*}}single{{.*}} slot 5 offset 0 : !sol.struct<"Single_{{[0-9]+}}", Storage, (!sol.func_ref<() -> !sol.struct<"Single_{{[0-9]+}}", Memory, {{.*}}>>)>
// A struct on a function cycle through a recursive struct stays literal, since that struct already
// breaks the cycle.
// CHECK:   sol.state_var @{{.*}}recursive{{.*}} slot 6 offset 0 : !sol.struct<"ArrayRecursive_{{[0-9]+}}", Storage, (!sol.array<? x !sol.struct<"ArrayRecursive_{{[0-9]+}}", Storage>, Storage>, !sol.func_ref<(!sol.struct<(!sol.func_ref<(!sol.struct<"ArrayRecursive_{{[0-9]+}}", Memory, {{.*}}>) -> ()>), Memory>) -> ()>, ui256)>
// CHECK:   sol.state_var @{{.*}}onCycle{{.*}} slot 9 offset 0 : !sol.struct<(!sol.func_ref<(!sol.struct<"ArrayRecursive_{{[0-9]+}}", Memory, {{.*}}>) -> ()>), Storage>
// CHECK:   sol.state_var @{{.*}}tail{{.*}} slot 10 offset 0 : ui256

// CHECK:   sol.func @{{.*}}run{{.*}}(%arg0: !sol.struct<"Closure_{{[0-9]+}}", Memory, {{.*}}) -> ui256
// CHECK:     %[[FIELD:.*]] = sol.gep %{{.*}}, %{{.*}} : !sol.struct<"Closure_{{[0-9]+}}", Memory, {{.*}}>, ui64, !sol.ptr<!sol.func_ref<(!sol.struct<"Closure_{{[0-9]+}}", Memory, {{.*}}>) -> ui256>, Memory>
// CHECK:     %[[CALLEE:.*]] = sol.load %[[FIELD]] : !sol.ptr<!sol.func_ref<{{.*}}>, Memory>, !sol.func_ref<(!sol.struct<"Closure_{{[0-9]+}}", Memory, {{.*}}>) -> ui256>
// CHECK:     sol.icall %[[CALLEE]](%{{.*}}) : !sol.func_ref<(!sol.struct<"Closure_{{[0-9]+}}", Memory, {{.*}}>) -> ui256>, (!sol.struct<"Closure_{{[0-9]+}}", Memory, {{.*}}>) -> ui256

struct Closure {
    function(Closure memory) internal pure returns (uint256) call;
}

struct ArrayIntoFunctionCycle {
    FunctionBackToArray[] xs;
}

struct FunctionBackToArray {
    function(ArrayIntoFunctionCycle memory) internal f;
}

struct ReachingFunctionCycle {
    ArrayIntoFunctionCycle inner;
}

struct Multi {
    function() internal pure returns (Multi memory, uint256) f;
}

struct Single {
    function() internal pure returns (Single memory) f;
}

struct OnFunctionCycle {
    function(ArrayRecursive memory) internal f;
}

struct ArrayRecursive {
    ArrayRecursive[] kids;
    function(OnFunctionCycle memory) internal g;
    uint256 v;
}

contract C {
    Closure closure;
    ArrayIntoFunctionCycle first;
    FunctionBackToArray second;
    ReachingFunctionCycle third;
    Multi multi;
    Single single;
    ArrayRecursive recursive;
    OnFunctionCycle onCycle;
    uint256 tail;

    function run(Closure memory c) internal pure returns (uint256) {
        return c.call(c);
    }
}
