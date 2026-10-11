//! { "modes": [ "E" ], "cases": [ {
//!     "name": "length",
//!     "inputs": [ { "method": "length", "calldata": [] } ],
//!     "expected": [ "0" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256[] array;

    function length() public returns (uint256) {
        array.push(array.length);
        return array[0];
    }
}
