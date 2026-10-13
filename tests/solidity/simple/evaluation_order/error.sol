//! { "modes": [ "E" ], "cases": [ {
//!     "name": "revert",
//!     "inputs": [ { "method": "revertError", "calldata": [] } ],
//!     "expected": { "return_data": [ "0xa1fba12300000000000000000000000000000000000000000000000000000000", "0x0000004000000000000000000000000000000000000000000000000000000000", "0x0000000100000000000000000000000000000000000000000000000000000000", "0x0000000100000000000000000000000000000000000000000000000000000000", "0x0000004d00000000000000000000000000000000000000000000000000000000" ], "exception": true }
//! }, {
//!     "name": "require",
//!     "inputs": [ { "method": "requireError", "calldata": [] } ],
//!     "expected": { "return_data": [ "0xa1fba12300000000000000000000000000000000000000000000000000000000", "0x0000004000000000000000000000000000000000000000000000000000000000", "0x0000000100000000000000000000000000000000000000000000000000000000", "0x0000000100000000000000000000000000000000000000000000000000000000", "0x0000004d00000000000000000000000000000000000000000000000000000000" ], "exception": true }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256[] array;

    error Copied(uint256[] values, uint256 length);

    function grow() internal returns (uint256) {
        array.push(77);
        return array.length;
    }

    function revertError() public {
        revert Copied(array, grow());
    }

    function requireError() public {
        require(false, Copied(array, grow()));
    }
}
