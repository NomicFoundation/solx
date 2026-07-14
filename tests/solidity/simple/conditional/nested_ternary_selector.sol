//! { "modes": [
//!     "Y >=0.8.0"
//! ], "cases": [ {
//!     "name": "outer_false_inner_false",
//!     "inputs": [
//!         {
//!             "method": "check",
//!             "calldata": [
//!                 "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x92b07f0c00000000000000000000000000000000000000000000000000000000"
//!     ]
//! } ] }

// Regression test for an EVM back-end stackification miscompilation.

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract SelectorTarget {
    function trueSelector() external {}
    function falseSelector() external {}
}

contract Test {
    function check(bool outer) public returns (bytes4) {
        bool[1] memory input; // input[0] == false
        return outer
            ? (new SelectorTarget()).trueSelector.selector
            : (input[0]
                ? (new SelectorTarget()).trueSelector
                : (new SelectorTarget()).falseSelector).selector;
    }
}
