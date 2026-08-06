// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*every_clause.*}}
// CHECK:   %[[STATUS:[^,]*]], %{{.*}} = sol.ext_call "{{.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256, try_call} : !sol.address, (ui256) -> (i1, ui256)
// CHECK:   sol.try %[[STATUS]] {
// CHECK:   } panic {
// CHECK-NEXT:   ^bb{{[0-9]+}}(%{{.*}}: ui256):
// CHECK:   } error {
// CHECK-NEXT:   ^bb{{[0-9]+}}(%{{.*}}: !sol.string<Memory>):
// CHECK:   } fallback {
// CHECK-NEXT:   ^bb{{[0-9]+}}(%{{.*}}: !sol.string<Memory>):

// CHECK: sol.func @{{.*error_only.*}}
// CHECK:   sol.try %{{.*}} {
// CHECK:   } panic {
// CHECK-NEXT:   } error {
// CHECK-NEXT:   ^bb{{[0-9]+}}(%{{.*}}: !sol.string<Memory>):
// CHECK:   } fallback {
// CHECK-NEXT:   }

// CHECK: sol.func @{{.*unbound_catch_all.*}}
// CHECK:   sol.try %{{.*}} {
// CHECK:   } panic {
// CHECK-NEXT:   } error {
// CHECK-NEXT:   } fallback {
// CHECK-NEXT:   sol.constant 0 : ui8

// CHECK: sol.func @{{.*empty_catch_all.*}}
// CHECK:   sol.try %{{.*}} {
// CHECK:   } panic {
// CHECK-NEXT:   } error {
// CHECK-NEXT:   } fallback {
// CHECK-NEXT:   sol.yield

// CHECK: sol.func @{{.*void_callee.*}}
// CHECK:   %[[STATUS:.*]] = sol.ext_call "{{.*v.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> (), try_call} : !sol.address, (ui256) -> i1
// CHECK:   sol.try %[[STATUS]] {

// CHECK: sol.func @{{.*view_callee.*}}
// CHECK:   sol.ext_call "{{.*g.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256, static_call, try_call} : !sol.address, (ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*named.*}}
// CHECK:   sol.constant 11 : ui8
// CHECK:   sol.constant 99 : ui8
// CHECK:   sol.ext_call "{{.*h.*}}"(%{{.*}}, %{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256, ui256) -> ui256, try_call} : !sol.address, (ui256, ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*value_option.*}}
// CHECK:   %[[VALUE:.*]] = sol.cast %c5_ui8 : ui8 to ui256
// CHECK:   %[[STATUS:[^,]*]], %{{.*}} = sol.ext_call "{{.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %[[VALUE]] selector %{{.*}} {callee_type = (ui256) -> ui256, try_call} : !sol.address, (ui256) -> (i1, ui256)
// CHECK:   sol.try %[[STATUS]] {

// CHECK: sol.func @{{.*multi_return.*}}
// CHECK:   %[[STATUS:[^,]*]], %[[OUTS:[^:]*]]:2 = sol.ext_call "{{.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> (ui256, ui256), try_call} : !sol.address, (ui256) -> (i1, ui256, ui256)
// CHECK:   sol.try %[[STATUS]] {
// CHECK:     sol.store %[[OUTS]]#0
// CHECK:     sol.store %[[OUTS]]#1

contract C {
    function every_clause(I i) public returns (uint256) {
        try i.f(1) returns (uint256 r) {
            return r;
        } catch Panic(uint256 code) {
            return code;
        } catch Error(string memory reason) {
            return bytes(reason).length;
        } catch (bytes memory payload) {
            return payload.length;
        }
    }

    function error_only(I i) public returns (uint256) {
        try i.f(2) returns (uint256 r) {
            return r;
        } catch Error(string memory reason) {
            return bytes(reason).length;
        }
    }

    function unbound_catch_all(I i) public returns (uint256) {
        try i.f(3) returns (uint256 r) {
            return r;
        } catch {
            return 0;
        }
    }

    function empty_catch_all(I i) public returns (uint256) {
        try i.f(3) returns (uint256 r) {
            return r;
        } catch {}
        return 0;
    }

    function void_callee(I i) public returns (uint256) {
        try i.v(4) {
            return 1;
        } catch {
            return 0;
        }
    }

    function view_callee(I i) public view returns (uint256) {
        try i.g(4) returns (uint256 r) {
            return r;
        } catch {
            return 0;
        }
    }

    function named(I i) public returns (uint256) {
        try i.h({b: 99, a: 11}) returns (uint256 r) {
            return r;
        } catch {
            return 0;
        }
    }

    function value_option(I i) public returns (uint256) {
        try i.p{value: 5}(6) returns (uint256 r) {
            return r;
        } catch {
            return 0;
        }
    }

    function multi_return(I i) public returns (uint256) {
        try i.q(7) returns (uint256 first, uint256 second) {
            return first - second;
        } catch {
            return 0;
        }
    }
}

interface I {
    function f(uint256 a) external returns (uint256);

    function v(uint256 a) external;

    function g(uint256 a) external view returns (uint256);

    function h(uint256 a, uint256 b) external returns (uint256);

    function p(uint256 a) external payable returns (uint256);

    function q(uint256 a) external returns (uint256, uint256);
}
