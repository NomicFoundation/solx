//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "zero_init_dirty_mem()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "null_ptr_bytes()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "96",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "null_ptr_string()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "zero_init_mixed()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0",
//!                         "0",
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "literal_bytes()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "5",
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "literal_string()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "3",
//!                         "5",
//!                         "4"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    struct WithBytes { uint256 x; bytes b; uint256 y; }
    struct WithString { uint256 x; string s; uint256 y; }
    struct Mixed { uint8 a; bytes b; uint8 c; string s; uint8 d; }

    // Dirty the struct's future memory slots to expose unwritten fields.
    // Before the offset-tracking fix, wb.y was never zeroed and returned 55.
    function zero_init_dirty_mem() public pure returns (uint256, uint256, uint256) {
        assembly {
            let p := mload(0x40)
            mstore(p,            42)
            mstore(add(p, 0x20), 99)
            mstore(add(p, 0x40), 55)
        }
        WithBytes memory wb;
        return (wb.x, wb.b.length, wb.y);
    }

    // Pollute keccak scratch space at address 0.
    // Before the null-pointer fix, wb.b pointed to 0, so wb.b.length read
    // mload(0) = 77. The bytes pointer itself must be the zero-sentinel 0x60.
    function null_ptr_bytes() public pure returns (uint256, uint256) {
        assembly { mstore(0, 77) }
        WithBytes memory wb;
        uint256 ptr;
        assembly {
            // wb is a pointer to the struct. x is at offset 0, b (pointer) at offset 32.
            ptr := mload(add(wb, 0x20))
        }
        return (ptr, wb.b.length);
    }

    // Same check for string fields.
    function null_ptr_string() public pure returns (uint256) {
        assembly { mstore(0, 77) }
        WithString memory ws;
        return bytes(ws.s).length;
    }

    // Mixed struct: every field at the correct offset, both bytes and string
    // sentinels set correctly.
    function zero_init_mixed() public pure returns (uint8, uint256, uint8, uint256, uint8) {
        assembly { mstore(0, 77) }
        Mixed memory m;
        return (m.a, m.b.length, m.c, bytes(m.s).length, m.d);
    }

    // Literal construction (!zeroInit path): all fields written explicitly.
    function literal_bytes() public pure returns (uint256, uint256, uint256) {
        WithBytes memory wb = WithBytes({x: 1, b: "hello", y: 2});
        return (wb.x, wb.b.length, wb.y);
    }

    function literal_string() public pure returns (uint256, uint256, uint256) {
        WithString memory ws = WithString({x: 3, s: "world", y: 4});
        return (ws.x, bytes(ws.s).length, ws.y);
    }
}
