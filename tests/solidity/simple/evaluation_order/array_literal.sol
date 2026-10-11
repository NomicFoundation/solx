//! { "modes": [ "E" ], "cases": [ {
//!     "name": "enum_element",
//!     "inputs": [ { "method": "enumElement", "calldata": [] } ],
//!     "expected": { "return_data": [ "0x4e487b7100000000000000000000000000000000000000000000000000000000", "0x0000002100000000000000000000000000000000000000000000000000000000" ], "exception": true }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    enum Choice { First, Second }

    function failChoice() internal pure returns (Choice) {
        revert("fail");
    }

    function enumElement() public pure returns (uint256) {
        Choice dirty;
        assembly {
            dirty := 2
        }
        Choice[2] memory choices = [dirty, failChoice()];
        return choices.length;
    }
}
