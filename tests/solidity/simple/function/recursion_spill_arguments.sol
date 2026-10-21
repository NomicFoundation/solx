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
//!         "96358159227711955972350532904688028991543876751521354589403931858356495278104"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

// The recursive function keeps its 12 arguments and 8 local values live
// across the recursive call. This forces the compiler to spill function
// arguments in a recursive function. The save of an argument's spill slot is
// fused with the argument's spill store.
pragma solidity >=0.8.0;
contract Test {
    function rec(
        uint256 n,
        uint256 a1, uint256 a2, uint256 a3, uint256 a4, uint256 a5, uint256 a6,
        uint256 a7, uint256 a8, uint256 a9, uint256 a10, uint256 a11, uint256 a12
    ) private pure returns (uint256) {
        if (n == 0) return a1 + a12;
        unchecked {
            uint256 v1 = a1 * 3 + a12; uint256 v2 = v1 * 5 + a11; uint256 v3 = v2 * 7 + a10;
            uint256 v4 = v3 * 11 + 4; uint256 v5 = v4 * 13 + 5; uint256 v6 = v5 * 17 + 6;
            uint256 v7 = v6 * 19 + 7; uint256 v8 = v7 * 23 + 8;
            uint256 r = rec(n - 1, v8, v7, v6, v5, v4, v3, v2, v1, n, v8 + n, v7 + n, v6 + n);
            uint256 s = r;
            s = s * 3 + a1; s = s * 3 + v1; s = s * 3 + a2; s = s * 3 + v2;
            s = s * 3 + a3; s = s * 3 + v3; s = s * 3 + a4; s = s * 3 + v4;
            s = s * 3 + a5; s = s * 3 + v5; s = s * 3 + a6; s = s * 3 + v6;
            s = s * 3 + a7; s = s * 3 + v7; s = s * 3 + a8; s = s * 3 + v8;
            s = s * 5 + a12; s = s * 5 + v8; s = s * 5 + a11; s = s * 5 + v7;
            s = s * 5 + a10; s = s * 5 + v6; s = s * 5 + a9; s = s * 5 + v5;
            s = s * 5 + a8; s = s * 5 + v4; s = s * 5 + a7; s = s * 5 + v3;
            s = s * 5 + a6; s = s * 5 + v2; s = s * 5 + a5; s = s * 5 + v1;
            s = s * 7 + a4; s = s * 7 + a3; s = s * 7 + a2; s = s * 7 + a1;
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
