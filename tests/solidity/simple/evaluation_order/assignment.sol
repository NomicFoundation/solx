//! { "modes": [ "E" ], "cases": [ {
//!     "name": "scalar",
//!     "inputs": [ { "method": "scalar", "calldata": [] } ],
//!     "expected": [ "2", "21" ]
//! }, {
//!     "name": "compound",
//!     "inputs": [ { "method": "compound", "calldata": [] } ],
//!     "expected": [ "5", "21" ]
//! }, {
//!     "name": "destructure",
//!     "inputs": [ { "method": "destructure", "calldata": [] } ],
//!     "expected": [ "40", "50", "4512" ]
//! }, {
//!     "name": "tuple_store",
//!     "inputs": [ { "method": "tupleStore", "calldata": [] } ],
//!     "expected": [ "1" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 sequence;
    mapping(uint256 => uint256) values;

    function leftIndex() internal returns (uint256) {
        sequence = sequence * 10 + 1;
        return 1;
    }

    function rightValue() internal returns (uint256) {
        sequence = sequence * 10 + 2;
        return 2;
    }

    function leftFirst() internal returns (uint256) {
        sequence = sequence * 10 + 1;
        return 1;
    }

    function leftSecond() internal returns (uint256) {
        sequence = sequence * 10 + 2;
        return 2;
    }

    function rightFirst() internal returns (uint256) {
        sequence = sequence * 10 + 4;
        return 40;
    }

    function rightSecond() internal returns (uint256) {
        sequence = sequence * 10 + 5;
        return 50;
    }

    function scalar() public returns (uint256, uint256) {
        sequence = 0;
        values[leftIndex()] = rightValue();
        return (values[1], sequence);
    }

    function compound() public returns (uint256, uint256) {
        values[1] = 3;
        sequence = 0;
        values[leftIndex()] += rightValue();
        return (values[1], sequence);
    }

    function destructure() public returns (uint256, uint256, uint256) {
        sequence = 0;
        (values[leftFirst()], values[leftSecond()]) = (rightFirst(), rightSecond());
        return (values[1], values[2], sequence);
    }

    function tupleStore() public pure returns (uint256) {
        uint256 value;
        (value, value) = (1, 2);
        return value;
    }
}
