//! { "cases": [ {
//!     "name": "base",
//!     "inputs": [
//!         {
//!             "method": "entry",
//!             "calldata": [
//!                 "0",
//!                 "5"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "21"
//!     ]
//! }, {
//!     "name": "shallow",
//!     "inputs": [
//!         {
//!             "method": "entry",
//!             "calldata": [
//!                 "1",
//!                 "5"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "36371223922747178097432"
//!     ]
//! }, {
//!     "name": "deep",
//!     "inputs": [
//!         {
//!             "method": "entry",
//!             "calldata": [
//!                 "3",
//!                 "5"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1583840839887893381608789339383715754276943747773118282903586483965"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

// The recursive function takes 12 arguments and computes 8 more values, so
// 20 values are live while the arguments of the recursive call are built.
// The last use of every one of them is an argument expression of the call,
// so they all die at the call. The 8 values defined after the call are
// consumed twice, in opposite orders. Nothing is live across the call, so
// every spill slot is call-free and needs no callee-save code.
// Twenty live values exceed the sixteen the EVM can reach on the stack.
pragma solidity >=0.8.0;
contract Test {
    function rec(
        uint256 n,
        uint256 a1, uint256 a2, uint256 a3, uint256 a4, uint256 a5, uint256 a6,
        uint256 a7, uint256 a8, uint256 a9, uint256 a10, uint256 a11, uint256 a12
    ) private pure returns (uint256) {
        unchecked {
            if (n == 0) return a1 + a12;
            uint256 v1 = a1 * 3 + 1; uint256 v2 = v1 * 5 + 2; uint256 v3 = v2 * 7 + 3;
            uint256 v4 = v3 * 11 + 4; uint256 v5 = v4 * 13 + 5; uint256 v6 = v5 * 17 + 6;
            uint256 v7 = v6 * 19 + 7; uint256 v8 = v7 * 23 + 8;
            uint256 r = rec(n - 1, v8 + a2, v7 + a3, v6 + a4, v5 + a5, v4 + a6, v3 + a7,
                            v2 + a8, v1 + a9, n + a10, v8 + a11, v7 + a12, v6 + n);
            uint256 w1 = r * 3 + 51; uint256 w2 = w1 * 5 + 52; uint256 w3 = w2 * 7 + 53;
            uint256 w4 = w3 * 11 + 54; uint256 w5 = w4 * 13 + 55; uint256 w6 = w5 * 17 + 56;
            uint256 w7 = w6 * 19 + 57; uint256 w8 = w7 * 23 + 58;
            uint256 s = r;
            s = s * 3 + w1; s = s * 3 + w2; s = s * 3 + w3; s = s * 3 + w4; s = s * 3 + w5;
            s = s * 3 + w6; s = s * 3 + w7; s = s * 3 + w8;
            s = s * 5 + w8; s = s * 5 + w7; s = s * 5 + w6; s = s * 5 + w5; s = s * 5 + w4;
            s = s * 5 + w3; s = s * 5 + w2; s = s * 5 + w1;
            return s;
        }
    }
    function entry(uint256 n, uint256 seed) public pure returns (uint256) {
        unchecked {
            return rec(n, seed, seed + 1, seed + 2, seed + 3, seed + 4, seed + 5,
                       seed + 6, seed + 7, seed + 8, seed + 9, seed + 10, seed + 11);
        }
    }
}
