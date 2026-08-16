// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Free functions reached only via a user-defined operator binding: solc's print-init calls
// @add yet never lowers it or its helper, so this is solx-only.

// CHECK: sol.func @{{.*}}f{{.*}}(%{{.*}}: si32, %{{.*}}: si32) -> si32
// CHECK:   sol.call @"add(T,T)_[[ADD:[0-9]+]]"(%{{.*}}, %{{.*}}) : (si32, si32) -> si32
// CHECK: sol.func @"add(T,T)_[[ADD]]"(%{{.*}}: si32, %{{.*}}: si32) -> si32
// CHECK:   sol.call @"helper(T)_[[HELPER:[0-9]+]]"(%{{.*}}) : (si32) -> si32
// CHECK: sol.func @"helper(T)_[[HELPER]]"(%{{.*}}: si32) -> si32

type T is int32;
using {add as +} for T global;

function helper(T x) pure returns (T) {
    return x;
}

function add(T x, T y) pure returns (T) {
    return helper(x);
}

contract C {
    function f(T x, T y) public pure returns (T) {
        return x + y;
    }
}
