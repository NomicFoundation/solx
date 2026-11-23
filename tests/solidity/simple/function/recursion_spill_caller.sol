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
//!         "4647288044333473633684030127507600222078589689075866825621380"
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
//!         "8012745388256867150828348975459606191851542952464111371533950618678185805947"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

// The non-recursive function entry keeps 18 values live across its call to
// the recursive function rec, which spills in every activation. The stack is
// compressed across a call to a recursive function, so the caller's values
// go to its spill slots and must survive every activation of the callee.
// Eighteen live values exceed the sixteen the EVM can reach on the stack.
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
        unchecked {
            uint256 c1 = seed * 3 + 301; uint256 c2 = c1 * 5 + 302; uint256 c3 = c2 * 7 + 303;
            uint256 c4 = c3 * 11 + 304; uint256 c5 = c4 * 13 + 305; uint256 c6 = c5 * 17 + 306;
            uint256 c7 = c6 * 19 + 307; uint256 c8 = c7 * 23 + 308; uint256 c9 = c8 * 29 + 309;
            uint256 c10 = c9 * 31 + 310; uint256 c11 = c10 * 37 + 311; uint256 c12 = c11 * 41 + 312;
            uint256 c13 = c12 * 43 + 313; uint256 c14 = c13 * 47 + 314; uint256 c15 = c14 * 53 + 315;
            uint256 c16 = c15 * 59 + 316; uint256 c17 = c16 * 61 + 317; uint256 c18 = c17 * 67 + 318;
            uint256 r = rec(n, c18);
            uint256 s = r;
            s = s * 7 + c1; s = s * 7 + c2; s = s * 7 + c3; s = s * 7 + c4; s = s * 7 + c5;
            s = s * 7 + c6; s = s * 7 + c7; s = s * 7 + c8; s = s * 7 + c9; s = s * 7 + c10;
            s = s * 7 + c11; s = s * 7 + c12; s = s * 7 + c13; s = s * 7 + c14; s = s * 7 + c15;
            s = s * 7 + c16; s = s * 7 + c17; s = s * 7 + c18;
            s = s * 11 + c18; s = s * 11 + c17; s = s * 11 + c16; s = s * 11 + c15; s = s * 11 + c14;
            s = s * 11 + c13; s = s * 11 + c12; s = s * 11 + c11; s = s * 11 + c10; s = s * 11 + c9;
            s = s * 11 + c8; s = s * 11 + c7; s = s * 11 + c6; s = s * 11 + c5; s = s * 11 + c4;
            s = s * 11 + c3; s = s * 11 + c2; s = s * 11 + c1;
            return s;
        }
    }
}
