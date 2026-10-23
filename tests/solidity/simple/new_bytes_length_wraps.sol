//! { "modes": [ "E" ], "cases": [ {
//!     "name": "length",
//!     "inputs": [
//!         {
//!             "method": "length",
//!             "calldata": [
//!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000004100000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! }, {
//!     "name": "overwrite",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000004100000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function length(uint256 n) external pure returns (uint256) {
        bytes memory a = new bytes(n);
        return a.length;
    }

    function overwrite(uint256 n) external pure returns (uint256) {
        bytes memory a = new bytes(n);
        uint256[] memory z = new uint256[](1);
        a[32] = 0xff;
        return z[0];
    }
}
