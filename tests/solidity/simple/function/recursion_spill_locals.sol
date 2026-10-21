//! { "cases": [ {
//!     "name": "base",
//!     "inputs": [
//!         {
//!             "method": "entry",
//!             "calldata": [
//!                 "0",
//!                 "7"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "7"
//!     ]
//! }, {
//!     "name": "deep",
//!     "inputs": [
//!         {
//!             "method": "entry",
//!             "calldata": [
//!                 "3",
//!                 "7"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "26116810937325380537420367466301745309681925340739941039410612194709442310876"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

// The recursive function keeps 18 local values live across the recursive
// call and consumes each of them twice, in opposite orders. This forces the
// compiler to spill in a recursive function. The spill slots are reused by
// every activation, so they must be callee-saved.
pragma solidity >=0.8.0;
contract Test {
    function rec(uint256 n, uint256 seed) private pure returns (uint256) {
        if (n == 0) return seed;
        unchecked {
            uint256 v1 = seed * 3 + 1; uint256 v2 = v1 * 5 + 2; uint256 v3 = v2 * 7 + 3;
            uint256 v4 = v3 * 11 + 4; uint256 v5 = v4 * 13 + 5; uint256 v6 = v5 * 17 + 6;
            uint256 v7 = v6 * 19 + 7; uint256 v8 = v7 * 23 + 8; uint256 v9 = v8 * 29 + 9;
            uint256 v10 = v9 * 31 + 10; uint256 v11 = v10 * 37 + 11; uint256 v12 = v11 * 41 + 12;
            uint256 v13 = v12 * 43 + 13; uint256 v14 = v13 * 47 + 14; uint256 v15 = v14 * 53 + 15;
            uint256 v16 = v15 * 59 + 16; uint256 v17 = v16 * 61 + 17; uint256 v18 = v17 * 67 + 18;
            uint256 r = rec(n - 1, v18);
            uint256 s = r;
            s = s * 3 + v1; s = s * 3 + v2; s = s * 3 + v3; s = s * 3 + v4; s = s * 3 + v5;
            s = s * 3 + v6; s = s * 3 + v7; s = s * 3 + v8; s = s * 3 + v9; s = s * 3 + v10;
            s = s * 3 + v11; s = s * 3 + v12; s = s * 3 + v13; s = s * 3 + v14; s = s * 3 + v15;
            s = s * 3 + v16; s = s * 3 + v17; s = s * 3 + v18;
            s = s * 5 + v18; s = s * 5 + v17; s = s * 5 + v16; s = s * 5 + v15; s = s * 5 + v14;
            s = s * 5 + v13; s = s * 5 + v12; s = s * 5 + v11; s = s * 5 + v10; s = s * 5 + v9;
            s = s * 5 + v8; s = s * 5 + v7; s = s * 5 + v6; s = s * 5 + v5; s = s * 5 + v4;
            s = s * 5 + v3; s = s * 5 + v2; s = s * 5 + v1;
            return s;
        }
    }
    function entry(uint256 n, uint256 seed) public pure returns (uint256) {
        return rec(n, seed);
    }
}
