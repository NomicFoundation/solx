//! { "modes": [ "E" ], "cases": [ {
//!     "name": "binary",
//!     "inputs": [ { "method": "binary", "calldata": [] } ],
//!     "expected": [ "5", "132" ]
//! }, {
//!     "name": "logical",
//!     "inputs": [ { "method": "logical", "calldata": [] } ],
//!     "expected": [ "3", "123" ]
//! }, {
//!     "name": "call",
//!     "inputs": [ { "method": "call", "calldata": [] } ],
//!     "expected": [ "23", "123" ]
//! }, {
//!     "name": "assignment",
//!     "inputs": [ { "method": "assignment", "calldata": [] } ],
//!     "expected": [ "2", "12" ]
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
        uint256 result = t(1) != 0 ? t(2) + t(3) : t(4);
        return (result, order);
    }

    function logical() public returns (uint256, uint256) {
        order = 0;
        uint256 result = t(1) != 0 && t(2) != 0 ? t(3) : t(4);
        return (result, order);
    }

    function call() public returns (uint256, uint256) {
        order = 0;
        uint256 result = t(1) != 0 ? pair(t(2), t(3)) : t(4);
        return (result, order);
    }

    function assignment() public returns (uint256, uint256) {
        order = 0;
        uint256 result = t(1) != 0 ? (store = t(2)) : (store = t(3));
        return (result, order);
    }
}
