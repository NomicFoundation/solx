// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// An unnamed return position reached without an explicit `return <value>`
// materialises its type's own default through `Value::type_default` /
// `Value::zero`. Each return type forces a distinct materialisation arm:
//   - scalar int/bool  -> `sol.constant`
//   - address          -> `sol.constant 0 : ui160` + `sol.address_cast` to address
//   - enum             -> `sol.constant 0 : ui256` + `sol.enum_cast` to the enum
//   - fixed bytes      -> `sol.constant 0 : uiN`   + `sol.bytes_cast`
//   - memory array     -> `sol.malloc zero_init`
//   - memory struct    -> `sol.malloc zero_init`
//   - memory fixed arr -> `sol.malloc zero_init`
//   - string / bytes   -> plain `sol.malloc` (a fresh zero-length buffer)
//
// solx materialises the default directly while solc round-trips it through an
// `sol.alloca`/`store`/`load` slot; the materialisation op itself is identical,
// so the checks pin only that op (plain CHECK, not CHECK-NEXT) and use CHECK-DAG
// because the two backends order functions differently.

contract C {
    enum E { A, B }
    struct S { uint256 x; }

    function uintDefault() public pure returns (uint256) {}
    function boolDefault() public pure returns (bool) {}
    function addrDefault() public pure returns (address) {}
    function enumDefault() public pure returns (E) {}
    function bytes4Default() public pure returns (bytes4) {}
    function dynArrDefault() public pure returns (uint256[] memory) {}
    function structDefault() public pure returns (S memory) {}
    function fixedArrDefault() public pure returns (uint256[2] memory) {}
    function stringDefault() public pure returns (string memory) {}
    function bytesDefault() public pure returns (bytes memory) {}
}

// CHECK-DAG: sol.func @{{.*uintDefault.*}}() -> ui256
// CHECK-DAG:   %{{.*}} = sol.constant 0 : ui256

// CHECK-DAG: sol.func @{{.*boolDefault.*}}() -> i1
// CHECK-DAG:   %{{.*}} = sol.constant false

// CHECK-DAG: sol.func @{{.*addrDefault.*}}() -> !sol.address
// CHECK-DAG:   %[[A:.*]] = sol.constant 0 : ui160
// CHECK-DAG:   %{{.*}} = sol.address_cast %[[A]] : ui160 to !sol.address

// CHECK-DAG: sol.func @{{.*enumDefault.*}}() -> !sol.enum<1>
// CHECK-DAG:   %[[EZ:.*]] = sol.constant 0 : ui256
// CHECK-DAG:   %{{.*}} = sol.enum_cast %[[EZ]] : ui256 to !sol.enum<1>

// CHECK-DAG: sol.func @{{.*bytes4Default.*}}() -> !sol.fixedbytes<4>
// CHECK-DAG:   %[[BZ:.*]] = sol.constant 0 : ui32
// CHECK-DAG:   %{{.*}} = sol.bytes_cast %[[BZ]] : ui32 to !sol.fixedbytes<4>

// CHECK-DAG: sol.func @{{.*dynArrDefault.*}}() -> !sol.array<? x ui256, Memory>
// CHECK-DAG:   %{{.*}} = sol.malloc zero_init :  !sol.array<? x ui256, Memory>

// CHECK-DAG: sol.func @{{.*structDefault.*}}() -> !sol.struct<(ui256), Memory>
// CHECK-DAG:   %{{.*}} = sol.malloc zero_init :  !sol.struct<(ui256), Memory>

// CHECK-DAG: sol.func @{{.*fixedArrDefault.*}}() -> !sol.array<2 x ui256, Memory>
// CHECK-DAG:   %{{.*}} = sol.malloc zero_init :  !sol.array<2 x ui256, Memory>

// CHECK-DAG: sol.func @{{.*stringDefault.*}}() -> !sol.string<Memory>
// CHECK-DAG: sol.func @{{.*bytesDefault.*}}() -> !sol.string<Memory>
