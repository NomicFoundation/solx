//! { "cases": [ {
//!     "name": "less",
//!     "inputs": [
//!         {
//!             "method": "f",
//!             "calldata": [ "1", "2" ]
//!         }
//!     ],
//!     "expected": [ "1" ]
//! }, {
//!     "name": "greater",
//!     "inputs": [
//!         {
//!             "method": "f",
//!             "calldata": [ "2", "1" ]
//!         }
//!     ],
//!     "expected": [ "0" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

// A pointer the constructor stores, which the runtime code retypes before calling it.
contract Test {
    function(address, address) pure returns (bool) x;

    constructor() {
        x = g;
    }

    function f(uint256 a, uint256 b) public view returns (bool) {
        function(address, address) pure returns (bool) y = x;
        function(uint256, uint256) pure returns (bool) z;
        assembly {
            z := y
        }
        return z(a, b);
    }

    function g(address a, address b) private pure returns (bool) {
        return a < b;
    }
}
