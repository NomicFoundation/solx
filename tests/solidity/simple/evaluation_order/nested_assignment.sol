//! { "modes": [ "E" ], "cases": [ {
//!     "name": "binary",
//!     "inputs": [ { "method": "binary", "calldata": [] } ],
//!     "expected": [ "5", "321" ]
//! }, {
//!     "name": "ternary",
//!     "inputs": [ { "method": "ternary", "calldata": [] } ],
//!     "expected": [ "3", "231" ]
//! }, {
//!     "name": "call",
//!     "inputs": [ { "method": "call", "calldata": [] } ],
//!     "expected": [ "23", "231" ]
//! }, {
//!     "name": "index_place",
//!     "inputs": [ { "method": "indexPlace", "calldata": [] } ],
//!     "expected": [ "3", "312" ]
//! }, {
//!     "name": "compound",
//!     "inputs": [ { "method": "compound", "calldata": [] } ],
//!     "expected": [ "105", "321" ]
//! }, {
//!     "name": "tuple",
//!     "inputs": [ { "method": "tuple", "calldata": [] } ],
//!     "expected": [ "3", "4", "3412" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 order;
    mapping(uint256 => uint256) slot;
    mapping(uint256 => mapping(uint256 => uint256)) grid;

    function t(uint256 n) internal returns (uint256) {
        order = order * 10 + n;
        return n;
    }

    function pair(uint256 x, uint256 y) internal pure returns (uint256) {
        return x * 10 + y;
    }

    function binary() public returns (uint256, uint256) {
        order = 0;
        slot[t(1)] = t(2) + t(3);
        return (slot[1], order);
    }

    function ternary() public returns (uint256, uint256) {
        order = 0;
        slot[t(1)] = t(2) != 0 ? t(3) : t(4);
        return (slot[1], order);
    }

    function call() public returns (uint256, uint256) {
        order = 0;
        slot[t(1)] = pair(t(2), t(3));
        return (slot[1], order);
    }

    function indexPlace() public returns (uint256, uint256) {
        order = 0;
        grid[t(1)][t(2)] = t(3);
        return (grid[1][2], order);
    }

    function compound() public returns (uint256, uint256) {
        slot[1] = 100;
        order = 0;
        slot[t(1)] += t(2) + t(3);
        return (slot[1], order);
    }

    function tuple() public returns (uint256, uint256, uint256) {
        order = 0;
        (slot[t(1)], slot[t(2)]) = (t(3), t(4));
        return (slot[1], slot[2], order);
    }
}
