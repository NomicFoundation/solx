//! { "cases": [ {
//!     "name": "counter",
//!     "inputs": [ { "method": "counter", "calldata": [] } ],
//!     "expected": [ "5" ]
//! }, {
//!     "name": "inclusive",
//!     "inputs": [ { "method": "inclusive", "calldata": [ "255" ] } ],
//!     "expected": {
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! }, {
//!     "name": "compound",
//!     "inputs": [ { "method": "compound", "calldata": [] } ],
//!     "expected": {
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! }, {
//!     "name": "decrement",
//!     "inputs": [ { "method": "decrement", "calldata": [] } ],
//!     "expected": {
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! }, {
//!     "name": "widened",
//!     "inputs": [ { "method": "widened", "calldata": [] } ],
//!     "expected": {
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! }, {
//!     "name": "body_write",
//!     "inputs": [ { "method": "bodyWrite", "calldata": [] } ],
//!     "expected": {
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function counter() public pure returns (uint8) {
        uint8 last = 0;
        for (uint8 i = 0; i < 5; ++i) {
            last = i + 1;
        }
        return last;
    }

    function inclusive(uint8 limit) public pure returns (uint8) {
        uint8 last = 0;
        for (uint8 i = 0; i <= limit; ++i) {
            last = i;
        }
        return last;
    }

    function compound() public pure returns (uint8) {
        uint8 last = 0;
        for (uint8 i = 250; i < 255; i += 10) {
            last = i;
        }
        return last;
    }

    function decrement() public pure returns (uint8) {
        uint8 last = 0;
        for (uint8 i = 0; i < 5; --i) {
            last = i;
        }
        return last;
    }

    function widened() public pure returns (uint8) {
        uint256 limit = 300;
        uint8 last = 0;
        for (uint8 i = 0; i < limit; ++i) {
            last = i;
        }
        return last;
    }

    function bodyWrite() public pure returns (uint8) {
        uint8 last = 0;
        for (uint8 i = 0; i < 5; ++i) {
            last = i;
            i = 255;
        }
        return last;
    }
}
