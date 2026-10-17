//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "g",
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
    struct R {
        uint256 a;
        R[] c;
    }

    function f(R calldata r) internal pure returns (uint256) {
        R memory m = r;
        return m.a;
    }

    function g() external pure returns (uint256) {
        function (R calldata) internal pure returns (uint256) p = f;
        return 1;
    }
}
