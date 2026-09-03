//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "st_s2s(uint256[4])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "10",
//!                         "20",
//!                         "30",
//!                         "40"
//!                     ],
//!                     "expected": [
//!                         "10",
//!                         "20",
//!                         "30",
//!                         "40"
//!                     ]
//!                 },
//!                 {
//!                     "method": "st_m2s(uint256[4])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "st_cd2s(uint256[4])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "50",
//!                         "60",
//!                         "70",
//!                         "80"
//!                     ],
//!                     "expected": [
//!                         "50",
//!                         "60",
//!                         "70",
//!                         "80"
//!                     ]
//!                 },
//!                 {
//!                     "method": "st_s2m(uint256[4])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "5",
//!                         "6",
//!                         "7",
//!                         "8"
//!                     ],
//!                     "expected": [
//!                         "5",
//!                         "6",
//!                         "7",
//!                         "8"
//!                     ]
//!                 },
//!                 {
//!                     "method": "st_m2m(uint256[4])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "st_cd2m(uint256[4])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "91",
//!                         "92",
//!                         "93",
//!                         "94"
//!                     ],
//!                     "expected": [
//!                         "91",
//!                         "92",
//!                         "93",
//!                         "94"
//!                     ]
//!                 },
//!                 {
//!                     "method": "st_self(uint256[4])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dyn_s2s(uint256[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "3",
//!                         "1",
//!                         "2",
//!                         "3"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "3",
//!                         "1",
//!                         "2",
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dyn_s2s_grow(uint256[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "3",
//!                         "7",
//!                         "8",
//!                         "9"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "3",
//!                         "7",
//!                         "8",
//!                         "9"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dyn_s2s_shrink(uint256[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "2",
//!                         "4",
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "2",
//!                         "4",
//!                         "5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dyn_m2s(uint256[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "3",
//!                         "21",
//!                         "22",
//!                         "23"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "3",
//!                         "21",
//!                         "22",
//!                         "23"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dyn_cd2s(uint256[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "3",
//!                         "41",
//!                         "42",
//!                         "43"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "3",
//!                         "41",
//!                         "42",
//!                         "43"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dyn_s2m(uint256[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "3",
//!                         "31",
//!                         "32",
//!                         "33"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "3",
//!                         "31",
//!                         "32",
//!                         "33"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dyn_m2m(uint256[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "3",
//!                         "1",
//!                         "2",
//!                         "3"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dyn_cd2m(uint256[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "3",
//!                         "61",
//!                         "62",
//!                         "63"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "3",
//!                         "61",
//!                         "62",
//!                         "63"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dyn_self(uint256[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "3",
//!                         "1",
//!                         "2",
//!                         "3"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "3",
//!                         "1",
//!                         "2",
//!                         "3"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Deep-copy tests covering all data-location combinations for 1-D uint arrays.
//
// Data-location combinations covered:
//   storage  → storage  (equal length, grow, shrink)
//   memory   → storage  /  storage → memory  /  memory → memory
//   calldata → storage  /  calldata → memory

contract Test {
    uint[4] sA;
    uint[4] sB;

    uint[] dA;
    uint[] dB;

    // Storage → Storage: dst must be an independent copy of src.
    function st_s2s(uint[4] memory src) public returns (uint[4] memory) {
        sA = src;
        sB = sA;
        sA[0] = 0;
        return sB;
    }

    // Memory → Storage
    function st_m2s(uint[4] memory src) public returns (uint[4] memory) {
        sA = src;
        return sA;
    }

    // CallData → Storage
    function st_cd2s(uint[4] calldata src) external returns (uint[4] memory) {
        sA = src;
        return sA;
    }

    // Storage → Memory: mutating storage must not affect the memory copy.
    function st_s2m(uint[4] memory src) public returns (uint[4] memory) {
        sA = src;
        uint[4] memory m = sA;
        sA[0] = 0;
        return m;
    }

    // Memory → Memory: alias — dst and src share the same memory block.
    function st_m2m(uint[4] memory src) public pure returns (bool) {
        uint[4] memory dst = src;
        dst[0] = 99;
        return src[0] == 99;
    }

    // CallData → Memory
    function st_cd2m(uint[4] calldata src) external pure returns (uint[4] memory) {
        uint[4] memory m = src;
        return m;
    }

    // Storage → Storage (self-assignment): array must be unchanged.
    function st_self(uint[4] memory src) public returns (uint[4] memory) {
        sA = src;
        sA = sA;
        return sA;
    }

    // Storage → Storage (equal length): dst must be an independent copy of src.
    function dyn_s2s(uint[] memory src) public returns (uint[] memory) {
        dA = new uint[](0);
        dB = new uint[](0);
        dA = src;
        dB.push(0); dB.push(0); dB.push(0);
        dB = dA;
        return dB;
    }

    // Storage → Storage (dst grows: 1 → src.length).
    function dyn_s2s_grow(uint[] memory src) public returns (uint[] memory) {
        dA = new uint[](0);
        dB = new uint[](0);
        dA = src;
        dB.push(0);
        dB = dA;
        return dB;
    }

    // Storage → Storage (dst shrinks: 4 → src.length; vacated slots zeroed).
    function dyn_s2s_shrink(uint[] memory src) public returns (uint[] memory) {
        dA = new uint[](0);
        dB = new uint[](0);
        dA = src;
        dB.push(0); dB.push(0); dB.push(0); dB.push(0);
        dB = dA;
        return dB;
    }

    // Memory → Storage
    function dyn_m2s(uint[] memory src) public returns (uint[] memory) {
        dA = new uint[](0);
        dA = src;
        return dA;
    }

    // CallData → Storage
    function dyn_cd2s(uint[] calldata src) external returns (uint[] memory) {
        dA = new uint[](0);
        dA = src;
        return dA;
    }

    // Storage → Memory: mutating storage must not affect the memory copy.
    function dyn_s2m(uint[] memory src) public returns (uint[] memory) {
        dA = new uint[](0);
        dA = src;
        uint[] memory m = dA;
        dA[0] = 0;
        return m;
    }

    // Memory → Memory: alias.
    function dyn_m2m(uint[] memory src) public pure returns (bool) {
        uint[] memory dst = src;
        dst[0] = 99;
        return src[0] == 99;
    }

    // CallData → Memory
    function dyn_cd2m(uint[] calldata src) external pure returns (uint[] memory) {
        uint[] memory m = src;
        return m;
    }

    // Storage → Storage (self-assignment): array must be unchanged.
    function dyn_self(uint[] memory src) public returns (uint[] memory) {
        dA = new uint[](0);
        dA = src;
        dA = dA;
        return dA;
    }
}
