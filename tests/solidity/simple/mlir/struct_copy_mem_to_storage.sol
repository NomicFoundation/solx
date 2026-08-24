//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "memToStorage()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "readSrcSlot0()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000007ffffffffffffffffff"
//!                     ]
//!                 },
//!                 {
//!                     "method": "readSrcSlot4()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x830080deadbeef03ffffffffffffffffffffffffffffffffffffffffffffffff"
//!                     ]
//!                 },
//!                 {
//!                     "method": "readSrcTyped()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "readSrcExt()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "memToStorageDirtyI72()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "readSrcSlot0()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000ffffffffffffffffff"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Memory-to-storage copy of S.
// Builds an S in memory, assigns it to a storage variable, and verifies
// the resulting packed storage slot bit layout via sload.
//
// After `src = m` (memory S → storage):
//   slot+0: i72=-1 (bits 0-71), u8=7 (bits 72-79)
//     → 0x0000000000000000000000000000000000000000000007ffffffffffffffffff
//
//   slot+4: i192=-1 (bits 0-191), u8b=3 (bits 192-199), b4=0xDEADBEEF (bits 200-231),
//           i8=-128/0x80 (bits 232-239), i16=-32000/0x8300 (bits 240-255)
//     → 0x830080deadbeef03ffffffffffffffffffffffffffffffffffffffffffffffff

struct S {
    int72        i72;
    uint8        u8;
    bool[][]     flags;
    address[2]   addrs;
    int192       i192;
    uint8        u8b;
    bytes4       b4;
    int8         i8;
    int16        i16;
    int128       i128;
    uint72       u72;
    uint128      u128;
    uint224      u224;
    bytes8       b8;
    bytes19      b19;
}

contract Test {
    S src;

    // Build an S in memory with garbage in the upper bits of the i72 slot and
    // copy to storage. The write-side AND(fieldMask) must strip the dirty bits
    // so that slot 0 contains only i72=-1 (bits 0-71) with u8=0 (bits 72-79).
    //   Dirty memory word at m+0x000:
    //     upper bytes = 0xDEADBEEF (bits 224-255 = garbage)
    //     lower 9 bytes = 0xffffffffffffffffff (i72=-1 raw bits 0-71)
    //   Expected slot 0 after copy:
    //     0x0000000000000000000000000000000000000000000000ffffffffffffffffff
    function memToStorageDirtyI72() external {
        S memory m;
        assembly {
            mstore(m, 0xDEADBEEF00000000000000000000000000000000000000ffffffffffffffffff)
        }
        src = m;
    }

    // Build S in memory and copy to storage.
    function memToStorage() external {
        S memory m;
        m.i72 = -1;
        m.u8 = 7;
        m.i192 = -1;
        m.u8b = 3;
        m.b4 = hex"DEADBEEF";
        m.i8 = -128;
        m.i16 = -32000;
        m.i128 = -111;
        m.u72 = 99;
        m.u128 = 111;
        m.u224 = 5678;
        m.b8 = hex"0102030405060708";
        m.b19 = hex"aabbccdd112233445566778899aabbccdd1122";
        src = m;
    }

    // Raw slot 0 (i72+u8 packed).
    function readSrcSlot0() external view returns (uint256 v) {
        assembly { v := sload(src.slot) }
    }

    // Raw slot 4 (i192+u8b+b4+i8+i16 packed).
    function readSrcSlot4() external view returns (uint256 v) {
        assembly { v := sload(add(src.slot, 4)) }
    }

    // Typed reads from storage.
    function readSrcTyped()
        external view
        returns (int72 i72, uint8 u8, int192 i192, uint8 u8b, bytes4 b4)
    {
        return (src.i72, src.u8, src.i192, src.u8b, src.b4);
    }

    function readSrcExt()
        external view
        returns (int8 i8, int16 i16, int128 i128,
                 uint72 u72, uint128 u128, uint224 u224,
                 bytes8 b8, bytes19 b19)
    {
        return (src.i8, src.i16, src.i128, src.u72, src.u128, src.u224, src.b8, src.b19);
    }
}
