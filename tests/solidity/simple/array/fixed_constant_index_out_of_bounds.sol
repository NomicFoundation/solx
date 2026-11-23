//! { "cases": [ {
//!     "name": "storage",
//!     "inputs": [
//!         {
//!             "method": "inStorage",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000003200000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! }, {
//!     "name": "storage_length",
//!     "inputs": [
//!         {
//!             "method": "inStorageByLength",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000003200000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! }, {
//!     "name": "memory",
//!     "inputs": [
//!         {
//!             "method": "inMemory",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000003200000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! }, {
//!     "name": "calldata",
//!     "inputs": [
//!         {
//!             "method": "inCalldata",
//!             "calldata": [
//!                 "1", "2", "3"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000003200000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 constant INDEX = 3;

    uint256[3] array;
    uint256 guard = 7;

    function inStorage() public returns(uint256) {
        array[INDEX] = 99;
        return guard;
    }

    function inStorageByLength() public returns(uint256) {
        array[array.length] = 99;
        return guard;
    }

    function inMemory() public pure returns(uint256) {
        uint256[3] memory data;
        return data[INDEX];
    }

    function inCalldata(uint256[3] calldata data) public pure returns(uint256) {
        return data[INDEX];
    }
}
