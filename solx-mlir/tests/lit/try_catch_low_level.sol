// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*g.*}}({{.*}}) -> ui256
// CHECK: sol.address_cast %{{.*}} : !sol.contract<{{.*}}> to !sol.address
// CHECK: sol.gasleft : ui256
// CHECK: sol.ext_call "{{.*}}"() at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = () -> ui256, try_call} : !sol.address, () -> (i1, ui256)
// CHECK: sol.try
// CHECK: panic {
// CHECK: ^bb0({{.*}}: ui256):
// CHECK: error {
// CHECK: ^bb0({{.*}}: !sol.string<Memory>):
// CHECK: fallback {
// CHECK: ^bb0({{.*}}: !sol.string<Memory>):

interface I {
    function f() external returns (uint256);
}

contract C {
    function g(I i) external returns (uint256) {
        try i.f() returns (uint256 v) {
            return v;
        } catch Error(string memory reason) {
            return bytes(reason).length;
        } catch Panic(uint256 code) {
            return code;
        } catch (bytes memory data) {
            return data.length;
        }
    }
}
