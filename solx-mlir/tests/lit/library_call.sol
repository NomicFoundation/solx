// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*qualified_plain.*}}
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   %[[ARG:.*]] = sol.load
// CHECK:   sol.ext_call "{{.*mutate.*}}"(%[[ARG]]) at %[[ADDR]] gas %{{.*}} value %c0_ui256 selector %c1899731083_ui256 {callee_type = (ui256) -> ui256, delegate_call, library_call} : !sol.address, (ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*qualified_static.*}}
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   %[[ARG:.*]] = sol.load
// CHECK:   sol.ext_call "{{.*observe.*}}"(%[[ARG]]) at %[[ADDR]] gas %{{.*}} value %c0_ui256 selector %c2147616019_ui256 {callee_type = (ui256) -> ui256, delegate_call, library_call, static_call} : !sol.address, (ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*qualified_try.*}}
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   %[[ARG:.*]] = sol.load
// CHECK:   %[[STATUS:[^,]*]], %{{.*}} = sol.ext_call "{{.*mutate.*}}"(%[[ARG]]) at %[[ADDR]] gas %{{.*}} value %c0_ui256 selector %c1899731083_ui256 {callee_type = (ui256) -> ui256, delegate_call, library_call, try_call} : !sol.address, (ui256) -> (i1, ui256)
// CHECK:   sol.try %[[STATUS]] {

// CHECK: sol.func @{{.*qualified_static_try.*}}
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   %[[ARG:.*]] = sol.load
// CHECK:   %[[STATUS:[^,]*]], %{{.*}} = sol.ext_call "{{.*observe.*}}"(%[[ARG]]) at %[[ADDR]] gas %{{.*}} value %c0_ui256 selector %c2147616019_ui256 {callee_type = (ui256) -> ui256, delegate_call, library_call, static_call, try_call} : !sol.address, (ui256) -> (i1, ui256)
// CHECK:   sol.try %[[STATUS]] {

// CHECK: sol.func @{{.*qualified_named.*}}
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   %[[ARG:.*]] = sol.load
// CHECK:   sol.constant 2 : ui8
// CHECK:   sol.ext_call "{{.*named.*}}"(%[[ARG]], %{{.*}}) at %[[ADDR]] gas %{{.*}} value %c0_ui256 selector %c3351715987_ui256 {callee_type = (ui256, ui256) -> ui256, delegate_call, library_call, static_call} : !sol.address, (ui256, ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*attached_plain.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.load
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   sol.ext_call "{{.*mutate.*}}"(%[[RECEIVER]]) at %[[ADDR]] gas %{{.*}} value %c0_ui256 selector %c1899731083_ui256 {callee_type = (ui256) -> ui256, delegate_call, library_call} : !sol.address, (ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*attached_static.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.load
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   sol.ext_call "{{.*observe.*}}"(%[[RECEIVER]]) at %[[ADDR]] gas %{{.*}} value %c0_ui256 selector %c2147616019_ui256 {callee_type = (ui256) -> ui256, delegate_call, library_call, static_call} : !sol.address, (ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*attached_try.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.load
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   %[[STATUS:[^,]*]], %{{.*}} = sol.ext_call "{{.*mutate.*}}"(%[[RECEIVER]]) at %[[ADDR]] gas %{{.*}} value %c0_ui256 selector %c1899731083_ui256 {callee_type = (ui256) -> ui256, delegate_call, library_call, try_call} : !sol.address, (ui256) -> (i1, ui256)
// CHECK:   sol.try %[[STATUS]] {

// CHECK: sol.func @{{.*attached_static_try.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.load
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   %[[STATUS:[^,]*]], %{{.*}} = sol.ext_call "{{.*observe.*}}"(%[[RECEIVER]]) at %[[ADDR]] gas %{{.*}} value %c0_ui256 selector %c2147616019_ui256 {callee_type = (ui256) -> ui256, delegate_call, library_call, static_call, try_call} : !sol.address, (ui256) -> (i1, ui256)
// CHECK:   sol.try %[[STATUS]] {

// CHECK: sol.func @{{.*attached_named.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.load
// CHECK:   sol.constant 3 : ui8
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   sol.ext_call "{{.*named.*}}"(%[[RECEIVER]], %{{.*}}) at %[[ADDR]] gas %{{.*}} value %c0_ui256 selector %c3351715987_ui256 {callee_type = (ui256, ui256) -> ui256, delegate_call, library_call, static_call} : !sol.address, (ui256, ui256) -> (i1, ui256)

library Lib {
    function mutate(uint256 a) external returns (uint256) {
        return a + 1;
    }

    function named(uint256 a, uint256 b) external pure returns (uint256) {
        return a - b;
    }

    function observe(uint256 a) external view returns (uint256) {
        return a + 2;
    }
}

contract User {
    using Lib for uint256;

    function qualified_plain(uint256 x) public returns (uint256) {
        return Lib.mutate(x);
    }

    function qualified_static(uint256 x) public view returns (uint256) {
        return Lib.observe(x);
    }

    function qualified_try(uint256 x) public returns (uint256) {
        try Lib.mutate(x) returns (uint256 r) { return r; } catch { return 0; }
    }

    function qualified_static_try(uint256 x) public view returns (uint256) {
        try Lib.observe(x) returns (uint256 r) { return r; } catch { return 0; }
    }

    function qualified_named(uint256 x) public pure returns (uint256) {
        return Lib.named({b: 2, a: x});
    }

    function attached_plain(uint256 x) public returns (uint256) {
        return x.mutate();
    }

    function attached_static(uint256 x) public view returns (uint256) {
        return x.observe();
    }

    function attached_try(uint256 x) public returns (uint256) {
        try x.mutate() returns (uint256 r) { return r; } catch { return 0; }
    }

    function attached_static_try(uint256 x) public view returns (uint256) {
        try x.observe() returns (uint256 r) { return r; } catch { return 0; }
    }

    function attached_named(uint256 x) public pure returns (uint256) {
        return x.named({b: 3});
    }
}
