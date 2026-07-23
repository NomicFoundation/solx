//! { "modes": [ "E" ], "cases": [ {
//!     "name": "binary",
//!     "inputs": [ { "method": "binary", "calldata": [] } ],
//!     "expected": [ "6", "321" ]
//! }, {
//!     "name": "ternary",
//!     "inputs": [ { "method": "ternary", "calldata": [] } ],
//!     "expected": [ "6", "412" ]
//! }, {
//!     "name": "assignment",
//!     "inputs": [ { "method": "assignment", "calldata": [] } ],
//!     "expected": [ "3", "21" ]
//! }, {
//!     "name": "call",
//!     "inputs": [ { "method": "call", "calldata": [] } ],
//!     "expected": [ "24", "231" ]
//! }, {
//!     "name": "index",
//!     "inputs": [ { "method": "index", "calldata": [] } ],
//!     "expected": [ "1", "21" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 order;
    mapping(uint256 => uint256) slot;
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
        uint256 result = (t(1) + t(2)) + t(3);
        return (result, order);
    }

    function ternary() public returns (uint256, uint256) {
        order = 0;
        uint256 result = (t(1) != 0 ? t(2) : t(3)) + t(4);
        return (result, order);
    }

    function assignment() public returns (uint256, uint256) {
        order = 0;
        uint256 result = t(1) + (store = t(2));
        return (result, order);
    }

    function call() public returns (uint256, uint256) {
        order = 0;
        uint256 result = t(1) + pair(t(2), t(3));
        return (result, order);
    }

    function index() public returns (uint256, uint256) {
        order = 0;
        uint256 result = t(1) + slot[t(2)];
        return (result, order);
    }
}
