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
//!     "name": "shallow",
//!     "inputs": [
//!         {
//!             "method": "entry",
//!             "calldata": [
//!                 "1",
//!                 "7"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "109096466465284927141987629097125777208602495391879610534499741947670356912240"
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
//!         "91601426646076623903469159605253839734836687671974246714012518314516544296429"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

// Both rec and other are recursive, but they belong to different call graph
// cycles. The function rec keeps 18 values live across its call to other and
// consumes them before its own recursive call. A call into another cycle can
// never re-enter the caller, so these spill slots need no callee-save code.
// The function other spills in its own activations meanwhile, so the test
// also checks that the spill areas of the two functions do not overlap.
// Eighteen live values exceed the sixteen the EVM can reach on the stack.
pragma solidity >=0.8.0;
contract Test {
    function other(uint256 n, uint256 seed) private pure returns (uint256) {
        if (n == 0) return seed;
        unchecked {
            uint256 w1 = seed * 3 + 201; uint256 w2 = w1 * 5 + 202; uint256 w3 = w2 * 7 + 203;
            uint256 w4 = w3 * 11 + 204; uint256 w5 = w4 * 13 + 205; uint256 w6 = w5 * 17 + 206;
            uint256 w7 = w6 * 19 + 207; uint256 w8 = w7 * 23 + 208; uint256 w9 = w8 * 29 + 209;
            uint256 w10 = w9 * 31 + 210; uint256 w11 = w10 * 37 + 211; uint256 w12 = w11 * 41 + 212;
            uint256 w13 = w12 * 43 + 213; uint256 w14 = w13 * 47 + 214; uint256 w15 = w14 * 53 + 215;
            uint256 w16 = w15 * 59 + 216; uint256 w17 = w16 * 61 + 217; uint256 w18 = w17 * 67 + 218;
            uint256 r = other(n - 1, w18);
            uint256 s = r;
            s = s * 3 + w1; s = s * 3 + w2; s = s * 3 + w3; s = s * 3 + w4; s = s * 3 + w5;
            s = s * 3 + w6; s = s * 3 + w7; s = s * 3 + w8; s = s * 3 + w9; s = s * 3 + w10;
            s = s * 3 + w11; s = s * 3 + w12; s = s * 3 + w13; s = s * 3 + w14; s = s * 3 + w15;
            s = s * 3 + w16; s = s * 3 + w17; s = s * 3 + w18;
            s = s * 5 + w18; s = s * 5 + w17; s = s * 5 + w16; s = s * 5 + w15; s = s * 5 + w14;
            s = s * 5 + w13; s = s * 5 + w12; s = s * 5 + w11; s = s * 5 + w10; s = s * 5 + w9;
            s = s * 5 + w8; s = s * 5 + w7; s = s * 5 + w6; s = s * 5 + w5; s = s * 5 + w4;
            s = s * 5 + w3; s = s * 5 + w2; s = s * 5 + w1;
            return s;
        }
    }
    function rec(uint256 n, uint256 seed) private pure returns (uint256) {
        if (n == 0) return seed;
        unchecked {
            uint256 v1 = seed * 3 + 1; uint256 v2 = v1 * 5 + 2; uint256 v3 = v2 * 7 + 3;
            uint256 v4 = v3 * 11 + 4; uint256 v5 = v4 * 13 + 5; uint256 v6 = v5 * 17 + 6;
            uint256 v7 = v6 * 19 + 7; uint256 v8 = v7 * 23 + 8; uint256 v9 = v8 * 29 + 9;
            uint256 v10 = v9 * 31 + 10; uint256 v11 = v10 * 37 + 11; uint256 v12 = v11 * 41 + 12;
            uint256 v13 = v12 * 43 + 13; uint256 v14 = v13 * 47 + 14; uint256 v15 = v14 * 53 + 15;
            uint256 v16 = v15 * 59 + 16; uint256 v17 = v16 * 61 + 17; uint256 v18 = v17 * 67 + 18;
            uint256 o = other(2, v18);
            uint256 t = o;
            t = t * 3 + v1; t = t * 3 + v2; t = t * 3 + v3; t = t * 3 + v4; t = t * 3 + v5;
            t = t * 3 + v6; t = t * 3 + v7; t = t * 3 + v8; t = t * 3 + v9; t = t * 3 + v10;
            t = t * 3 + v11; t = t * 3 + v12; t = t * 3 + v13; t = t * 3 + v14; t = t * 3 + v15;
            t = t * 3 + v16; t = t * 3 + v17; t = t * 3 + v18;
            t = t * 5 + v18; t = t * 5 + v17; t = t * 5 + v16; t = t * 5 + v15; t = t * 5 + v14;
            t = t * 5 + v13; t = t * 5 + v12; t = t * 5 + v11; t = t * 5 + v10; t = t * 5 + v9;
            t = t * 5 + v8; t = t * 5 + v7; t = t * 5 + v6; t = t * 5 + v5; t = t * 5 + v4;
            t = t * 5 + v3; t = t * 5 + v2; t = t * 5 + v1;
            uint256 r = rec(n - 1, t);
            return r * 3 + t;
        }
    }
    function entry(uint256 n, uint256 seed) public pure returns (uint256) {
        return rec(n, seed);
    }
}
