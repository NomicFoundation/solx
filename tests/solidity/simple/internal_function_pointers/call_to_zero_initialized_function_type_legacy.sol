//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "t",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000005100000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! } ], "toolchains": ["solx"] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

// This tests is an addendum to `tests/solidity/ethereum/functionTypes/call_to_zero_initialized_function_type_legacy.sol`
// and covers the return data for Solidity >=0.8 where the error is not empty.

contract Other {
    function addTwo(uint256 x) public pure returns (uint256) {
        return x + 2;
    }
}

contract Test {
    function (function (uint) external returns (uint)) internal returns (uint) ev;
    function (uint) external returns (uint) x;

    function store(function(uint) external returns (uint) y) public {
         x = y;
    }

    function eval(function(uint) external returns (uint) y) public returns (uint) {
        return y(7);
    }

    function t() public returns (uint256) {
        this.store((new Other()).addTwo);
        // This call panics
        return ev(x);
    }
}
