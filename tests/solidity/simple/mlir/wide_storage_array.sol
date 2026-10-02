//! { "modes": [ "E" ], "cases": [ {
//!     "name": "layout",
//!     "inputs": [ { "method": "slots", "calldata": [] } ],
//!     "expected": [
//!         "18446744073709551616",
//!         "18446744073709551617",
//!         "36893488147419103235",
//!         "55340232221128654851"
//!     ]
//! }, {
//!     "name": "struct_member_after_wide_array",
//!     "inputs": [ { "method": "structTail", "calldata": [] } ],
//!     "expected": [ "7" ]
//! }, {
//!     "name": "last_element",
//!     "inputs": [ { "method": "element", "calldata": [ "18446744073709551615" ] } ],
//!     "expected": [ "5" ]
//! }, {
//!     "name": "out_of_bounds",
//!     "inputs": [ { "method": "element", "calldata": [ "18446744073709551616" ] } ],
//!     "expected": {
//!         "return_data": [
//!             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000003200000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    struct Wide {
        uint256 head;
        uint256[2**64] body;
        uint256 tail;
    }

    uint256[2**64] wide;
    uint256 afterWide;
    Wide wideStruct;
    uint256[2][2**63] pairs;
    uint256 afterPairs;

    function slots() public pure returns (uint256 a, uint256 b, uint256 c, uint256 d) {
        assembly {
            a := afterWide.slot
            b := wideStruct.slot
            c := pairs.slot
            d := afterPairs.slot
        }
    }

    function structTail() public returns (uint256 value) {
        wideStruct.tail = 7;
        assembly {
            value := sload(36893488147419103234)
        }
    }

    function element(uint256 i) public returns (uint256) {
        wide[2**64 - 1] = 5;
        return wide[i];
    }
}
