//! { "cases": [ {
//!     "name": "a",
//!     "inputs": [
//!         {
//!             "method": "a",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 r;
    uint256 n;

    function f(uint256 a0, uint256 a1) internal {
        if (a0 == 7) {
            r = 1;
            return;
        }
        n += 1;
        if (n == 5) {
            r = 2;
            return;
        }
        f(a1, 7);
    }

    function a() external returns (uint256) {
        f(0, 0);
        return r;
    }
}
