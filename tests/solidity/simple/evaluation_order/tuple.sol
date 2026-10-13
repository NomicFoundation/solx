//! { "modes": [ "E" ], "cases": [ {
//!     "name": "declaration",
//!     "inputs": [ { "method": "declaration", "calldata": [] } ],
//!     "expected": [ "1", "1" ]
//! }, {
//!     "name": "return",
//!     "inputs": [ { "method": "returnStatement", "calldata": [] } ],
//!     "expected": [ "64", "1", "1", "77" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256[] array;

    function grow() internal returns (uint256[] storage) {
        array.push(77);
        return array;
    }

    function declaration() public returns (uint256, bool) {
        (uint256[] memory first, uint256[] memory second) = (array, grow());
        bool secondFirst;
        assembly {
            secondFirst := lt(second, first)
        }
        return (first.length, secondFirst);
    }

    function returnStatement() public returns (uint256[] memory, uint256) {
        return (array, grow().length);
    }
}
