//! { "modes": [ "E" ], "cases": [ {
//!     "name": "binary",
//!     "inputs": [ { "method": "binary", "calldata": [] } ],
//!     "expected": [ "1", "213" ]
//! }, {
//!     "name": "ternary",
//!     "inputs": [ { "method": "ternary", "calldata": [] } ],
//!     "expected": [ "1", "123" ]
//! }, {
//!     "name": "assignment",
//!     "inputs": [ { "method": "assignment", "calldata": [] } ],
//!     "expected": [ "1", "12" ]
//! }, {
//!     "name": "logical",
//!     "inputs": [ { "method": "logical", "calldata": [] } ],
//!     "expected": [ "1", "12" ]
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

    function binary() public returns (uint256, uint256) {
        order = 0;
        bool result = t(1) + t(2) != 0 && t(3) != 0;
        return (result ? 1 : 0, order);
    }

    function ternary() public returns (uint256, uint256) {
        order = 0;
        bool result = t(1) != 0 && (t(2) != 0 ? t(3) : 0) != 0;
        return (result ? 1 : 0, order);
    }

    function assignment() public returns (uint256, uint256) {
        order = 0;
        bool result = t(1) != 0 && (store = t(2)) != 0;
        return (result ? 1 : 0, order);
    }

    function logical() public returns (uint256, uint256) {
        order = 0;
        bool result = t(1) != 0 && (t(2) != 0 || t(3) != 0);
        return (result ? 1 : 0, order);
    }
}
