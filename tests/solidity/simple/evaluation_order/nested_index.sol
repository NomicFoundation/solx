//! { "modes": [ "E" ], "cases": [ {
//!     "name": "binary",
//!     "inputs": [ { "method": "binary", "calldata": [] } ],
//!     "expected": [ "0", "213" ]
//! }, {
//!     "name": "ternary",
//!     "inputs": [ { "method": "ternary", "calldata": [] } ],
//!     "expected": [ "0", "124" ]
//! }, {
//!     "name": "call",
//!     "inputs": [ { "method": "call", "calldata": [] } ],
//!     "expected": [ "0", "123" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 order;
    mapping(uint256 => mapping(uint256 => uint256)) grid;

    function t(uint256 n) internal returns (uint256) {
        order = order * 10 + n;
        return n;
    }

    function first(uint256 x, uint256 y) internal pure returns (uint256) {
        y;
        return x;
    }

    function binary() public returns (uint256, uint256) {
        order = 0;
        uint256 result = grid[t(1) + t(2)][t(3)];
        return (result, order);
    }

    function ternary() public returns (uint256, uint256) {
        order = 0;
        uint256 result = grid[t(1) != 0 ? t(2) : t(3)][t(4)];
        return (result, order);
    }

    function call() public returns (uint256, uint256) {
        order = 0;
        uint256 result = grid[first(t(1), t(2))][t(3)];
        return (result, order);
    }
}
