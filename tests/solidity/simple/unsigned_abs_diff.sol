//! { "cases": [ {
//!     "name": "f0",
//!     "inputs": [
//!         {
//!             "method": "f0",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff5"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 s = 3;

    function f0() external view returns (uint256) {
        unchecked {
            uint256 p = s % 6;
            uint256 q = (s / 7) % 6;
            uint256 t = p > q ? p - q : q - p;
            uint256 a = ~t;
            uint256 k = q + 7;
            return a > k ? a - k : k - a;
        }
    }
}
