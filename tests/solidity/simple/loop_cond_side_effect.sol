//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "trigger",
//!             "calldata": [
//!                 "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                 "0x0000000000000000000000000000000000000000000000000000000000000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    int256 state_T;

    function cond_fn(int256 a, int256 b) internal returns (bool) {
        state_T = 0;
        return a > b;
    }

    function trigger(int256 a, int256 b) external returns (int256) {
        int256 r;
        for (uint256 i = 0; cond_fn(a, b) && i == 0; i++) {
            int256[1] memory arr;
            r = 1;
        }
        for (uint256 i = 0; !cond_fn(a, b) && i == 0; i++) {
            r = 0;
        }
        return r;
    }
}
