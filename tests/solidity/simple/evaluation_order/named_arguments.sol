//! { "modes": [ "E" ], "cases": [ {
//!     "name": "call",
//!     "inputs": [ { "method": "call", "calldata": [] } ],
//!     "expected": [ "123", "123" ]
//! }, {
//!     "name": "struct_constructor",
//!     "inputs": [ { "method": "struct_constructor", "calldata": [] } ],
//!     "expected": [ "123", "123" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    struct S {
        uint256 a;
        uint256 b;
        uint256 c;
    }

    uint256 order;

    function t(uint256 n) internal returns (uint256) {
        order = order * 10 + n;
        return n;
    }

    function triple(uint256 a, uint256 b, uint256 c) internal pure returns (uint256) {
        return a * 100 + b * 10 + c;
    }

    function call() public returns (uint256, uint256) {
        order = 0;
        uint256 result = triple({c: t(3), a: t(1), b: t(2)});
        return (result, order);
    }

    function struct_constructor() public returns (uint256, uint256) {
        order = 0;
        S memory s = S({c: t(3), a: t(1), b: t(2)});
        return (s.a * 100 + s.b * 10 + s.c, order);
    }
}
