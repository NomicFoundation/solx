// RUN: solx --emit-mlir=sol %s | FileCheck %s

// `sol.bytes_cast` connects a fixed-bytes type only with its matching-width
// integer (`fixedbytes<N>` <-> `ui(N*8)`, `byte` <-> `ui8`). A conversion to a
// different-width integer must bridge through that partner integer rather than
// emit an ill-typed direct `bytes_cast`. Here a `bytes` element (a `byte`)
// compared as `ui256` lowers to `bytes_cast` to `ui8`, then a `ui8 -> ui256`
// integer widen — never `bytes_cast ... to ui256`.
//
// solx-only: solc compares the operands as `fixedbytes<1>` rather than widening
// to `ui256`, so the IR shapes differ; behavioural parity is covered by the
// tester (array/push/byte_array_push.sol).

// CHECK: sol.bytes_cast %{{[0-9]+}} : !sol.fixedbytes<1> to ui8
// CHECK: sol.cast %{{[0-9]+}} : ui8 to ui256

contract C {
    bytes data;

    function f() public returns (bool) {
        data.push(0x05);
        return data[0] == 0x05;
    }
}
