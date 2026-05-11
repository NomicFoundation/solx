//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "42"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Callee {
    function f(
        bool, bool, bool, bool, bool,
        bool, bool, bool, bool, bool,
        bool, bool, bool, bool, bool
    ) external pure returns (uint256) {
        return 42;
    }
}

contract Test {
    function test() external returns (uint256) {
        Callee c = new Callee();
        return c.f(
            false, false, false, false, false,
            false, false, false, false, false,
            false, false, false, false, false
        );
    }
}
