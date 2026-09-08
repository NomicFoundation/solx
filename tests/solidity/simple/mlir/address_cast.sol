//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "to_payable(address)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "from_payable(address)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "from_uint(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x10000000000000000000000000000000000000001"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "eq(address,address)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "eq(address,address)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0x2"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "from_bytes20(bytes20)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0100000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0x0100000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "to_contract(address)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "from_contract(address)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "this_to_address()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "Test.address"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function to_payable(address a) external pure returns (address payable) {
        return payable(a);
    }

    function from_payable(address payable a) external pure returns (address) {
        return address(a);
    }

    function from_uint(uint256 x) external pure returns (address) {
        return address(uint160(x));
    }

    function eq(address a, address b) external pure returns (bool) {
        return a == b;
    }

    function to_bytes20(address a) external pure returns (bytes20) {
        return bytes20(a);
    }

    function from_bytes20(bytes20 b) external pure returns (address) {
        return address(b);
    }

    function to_contract(address a) external pure returns (Test) {
        return Test(a);
    }

    function from_contract(Test c) external pure returns (address) {
        return address(c);
    }

    function this_to_address() public returns (address) {
      return address(this);
    }
}
