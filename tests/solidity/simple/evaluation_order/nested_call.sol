//! { "modes": [ "E" ], "cases": [ {
//!     "name": "binary",
//!     "inputs": [ { "method": "binary", "calldata": [] } ],
//!     "expected": [ "33", "213" ]
//! }, {
//!     "name": "ternary",
//!     "inputs": [ { "method": "ternary", "calldata": [] } ],
//!     "expected": [ "24", "124" ]
//! }, {
//!     "name": "assignment",
//!     "inputs": [ { "method": "assignment", "calldata": [] } ],
//!     "expected": [ "12", "12" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 order;
    uint256 store;

    function t(uint256 n) internal returns (uint256) {
        order = order * 10 + n;
        return n;
    }

    function pair(uint256 x, uint256 y) internal pure returns (uint256) {
        return x * 10 + y;
    }

    function binary() public returns (uint256, uint256) {
        order = 0;
        uint256 result = pair(t(1) + t(2), t(3));
        return (result, order);
    }

    function ternary() public returns (uint256, uint256) {
        order = 0;
        uint256 result = pair(t(1) != 0 ? t(2) : t(3), t(4));
        return (result, order);
    }

    function assignment() public returns (uint256, uint256) {
        order = 0;
        uint256 result = pair((store = t(1)), t(2));
        return (result, order);
    }
}
