//! { "modes": [ "E" ], "cases": [ {
//!     "name": "logical_and",
//!     "inputs": [ { "method": "logicalAnd", "calldata": [] } ],
//!     "expected": [ "1", "43" ]
//! }, {
//!     "name": "logical_and_short_circuit",
//!     "inputs": [ { "method": "logicalAndShortCircuit", "calldata": [] } ],
//!     "expected": [ "0", "2" ]
//! }, {
//!     "name": "logical_or_short_circuit",
//!     "inputs": [ { "method": "logicalOrShortCircuit", "calldata": [] } ],
//!     "expected": [ "1", "4" ]
//! }, {
//!     "name": "logical_or_fallback",
//!     "inputs": [ { "method": "logicalOrFallback", "calldata": [] } ],
//!     "expected": [ "1", "23" ]
//! }, {
//!     "name": "conditional_true",
//!     "inputs": [ { "method": "conditionalTrue", "calldata": [] } ],
//!     "expected": [ "3", "43" ]
//! }, {
//!     "name": "conditional_false",
//!     "inputs": [ { "method": "conditionalFalse", "calldata": [] } ],
//!     "expected": [ "5", "25" ]
//! }, {
//!     "name": "nested_index",
//!     "inputs": [ { "method": "nestedIndex", "calldata": [] } ],
//!     "expected": [ "9", "43" ]
//! }, {
//!     "name": "function_call",
//!     "inputs": [ { "method": "functionCall", "calldata": [] } ],
//!     "expected": [ "435", "435" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 sequence;
    mapping(uint256 => mapping(uint256 => uint256)) nested;

    function left() internal returns (uint256) {
        sequence = sequence * 10 + 4;
        return 4;
    }

    function right() internal returns (uint256) {
        sequence = sequence * 10 + 3;
        return 3;
    }

    function modulus() internal returns (uint256) {
        sequence = sequence * 10 + 5;
        return 5;
    }

    function falseCondition() internal returns (bool) {
        sequence = sequence * 10 + 2;
        return false;
    }

    function combine(uint256 a, uint256 b, uint256 c) internal pure returns (uint256) {
        return a * 100 + b * 10 + c;
    }

    function logicalAnd() public returns (bool, uint256) {
        sequence = 0;
        bool result = left() != 0 && right() != 0;
        return (result, sequence);
    }

    function logicalAndShortCircuit() public returns (bool, uint256) {
        sequence = 0;
        bool result = falseCondition() && right() != 0;
        return (result, sequence);
    }

    function logicalOrShortCircuit() public returns (bool, uint256) {
        sequence = 0;
        bool result = left() != 0 || right() != 0;
        return (result, sequence);
    }

    function logicalOrFallback() public returns (bool, uint256) {
        sequence = 0;
        bool result = falseCondition() || right() != 0;
        return (result, sequence);
    }

    function conditionalTrue() public returns (uint256, uint256) {
        sequence = 0;
        uint256 result = left() != 0 ? right() : modulus();
        return (result, sequence);
    }

    function conditionalFalse() public returns (uint256, uint256) {
        sequence = 0;
        uint256 result = falseCondition() ? right() : modulus();
        return (result, sequence);
    }

    function nestedIndex() public returns (uint256, uint256) {
        nested[4][3] = 9;
        sequence = 0;
        uint256 result = nested[left()][right()];
        return (result, sequence);
    }

    function functionCall() public returns (uint256, uint256) {
        sequence = 0;
        uint256 result = combine(left(), right(), modulus());
        return (result, sequence);
    }
}
