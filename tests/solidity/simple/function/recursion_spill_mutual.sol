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
//!         "29"
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
//!         "1726943768027236365222580519114677648936688263102862929397351"
//!     ]
//! }, {
//!     "name": "deep",
//!     "inputs": [
//!         {
//!             "method": "entry",
//!             "calldata": [
//!                 "4",
//!                 "7"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "24414204256934313701338865033478412630583275525177173957436467874813123243153"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

// The functions f and g call each other, so they belong to one call graph
// cycle. Each keeps 18 values live across the call into the other one and
// consumes them twice, in opposite orders. A call into the same cycle may
// re-enter the caller, so the spill slots of both functions must be
// callee-saved, exactly as for a self-recursive call. The entry point calls
// both functions, so that neither is inlined into the other.
// Eighteen live values exceed the sixteen the EVM can reach on the stack.
pragma solidity >=0.8.0;
contract Test {
    function f(uint256 n, uint256 seed) private pure returns (uint256) {
        if (n == 0) return seed;
        unchecked {
            uint256 v1 = seed * 3 + 1; uint256 v2 = v1 * 5 + 2; uint256 v3 = v2 * 7 + 3;
            uint256 v4 = v3 * 11 + 4; uint256 v5 = v4 * 13 + 5; uint256 v6 = v5 * 17 + 6;
            uint256 v7 = v6 * 19 + 7; uint256 v8 = v7 * 23 + 8; uint256 v9 = v8 * 29 + 9;
            uint256 v10 = v9 * 31 + 10; uint256 v11 = v10 * 37 + 11; uint256 v12 = v11 * 41 + 12;
            uint256 v13 = v12 * 43 + 13; uint256 v14 = v13 * 47 + 14; uint256 v15 = v14 * 53 + 15;
            uint256 v16 = v15 * 59 + 16; uint256 v17 = v16 * 61 + 17; uint256 v18 = v17 * 67 + 18;
            uint256 r = g(n - 1, v18);
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
    function g(uint256 n, uint256 seed) private pure returns (uint256) {
        unchecked {
            if (n == 0) return seed + 1;
            uint256 w1 = seed * 3 + 101; uint256 w2 = w1 * 5 + 102; uint256 w3 = w2 * 7 + 103;
            uint256 w4 = w3 * 11 + 104; uint256 w5 = w4 * 13 + 105; uint256 w6 = w5 * 17 + 106;
            uint256 w7 = w6 * 19 + 107; uint256 w8 = w7 * 23 + 108; uint256 w9 = w8 * 29 + 109;
            uint256 w10 = w9 * 31 + 110; uint256 w11 = w10 * 37 + 111; uint256 w12 = w11 * 41 + 112;
            uint256 w13 = w12 * 43 + 113; uint256 w14 = w13 * 47 + 114; uint256 w15 = w14 * 53 + 115;
            uint256 w16 = w15 * 59 + 116; uint256 w17 = w16 * 61 + 117; uint256 w18 = w17 * 67 + 118;
            uint256 r = f(n - 1, w18);
            uint256 s = r;
            s = s * 7 + w1; s = s * 7 + w2; s = s * 7 + w3; s = s * 7 + w4; s = s * 7 + w5;
            s = s * 7 + w6; s = s * 7 + w7; s = s * 7 + w8; s = s * 7 + w9; s = s * 7 + w10;
            s = s * 7 + w11; s = s * 7 + w12; s = s * 7 + w13; s = s * 7 + w14; s = s * 7 + w15;
            s = s * 7 + w16; s = s * 7 + w17; s = s * 7 + w18;
            s = s * 11 + w18; s = s * 11 + w17; s = s * 11 + w16; s = s * 11 + w15; s = s * 11 + w14;
            s = s * 11 + w13; s = s * 11 + w12; s = s * 11 + w11; s = s * 11 + w10; s = s * 11 + w9;
            s = s * 11 + w8; s = s * 11 + w7; s = s * 11 + w6; s = s * 11 + w5; s = s * 11 + w4;
            s = s * 11 + w3; s = s * 11 + w2; s = s * 11 + w1;
            return s;
        }
    }
    function entry(uint256 n, uint256 seed) public pure returns (uint256) {
        unchecked {
            return f(n, seed) * 3 + g(n, seed);
        }
    }
}
