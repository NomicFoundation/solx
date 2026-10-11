//! { "modes": [ "E" ], "cases": [ {
//!     "name": "call",
//!     "inputs": [ { "method": "call", "calldata": [] } ],
//!     "expected": [ "123", "123" ]
//! }, {
//!     "name": "struct_constructor",
//!     "inputs": [ { "method": "struct_constructor", "calldata": [] } ],
//!     "expected": [ "123", "123" ]
//! }, {
//!     "name": "emit_event",
//!     "inputs": [ { "method": "emit_event", "calldata": [] } ],
//!     "expected": { "return_data": [ "312" ], "events": [ { "topics": [ "0x0669a7711ee30624d1b9d0b20068dd5dea4f4e4c3d0a1f504945d8ac3bbf4c1f", "3" ], "values": [ "1", "2" ] } ] }
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

    event Triple(uint256 a, uint256 b, uint256 indexed c);

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

    function emit_event() public returns (uint256) {
        order = 0;
        emit Triple({b: t(2), c: t(3), a: t(1)});
        return order;
    }
}
