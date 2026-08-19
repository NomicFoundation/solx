//! { "cases": [ {
//!     "name": "zero_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "zero_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "433478394034343"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "121563127839120", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "433478394034343"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "433478394034343"
//!     ]
//! }, {
//!     "name": "ordinary_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "121563127839120", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "121563127839120"
//!     ]
//! }, {
//!     "name": "ordinary_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "451167890987656789", "3456789009876545678"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1559592207175311407777037810745307942"
//!     ]
//! }, {
//!     "name": "ordinary_ordinary_to_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x00000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0x0000000000000000000000000000000100000000000000000000000000000001"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
//!     ]
//! }, {
//!     "name": "ordinary_ordinary_min_overflow",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x0000002000000000000000000000000000000000000000000000000000000000", "0x0000000000000000000000000000000000000000000000000000000008000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_ordinary_overflow",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "2345678909876543212345678909876543212345678909876543", "23456789098765432123456789876543212345678987654321"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "109870817467274783132168917047078478060902558898874809919449649221827584233615"
//!     ]
//! }, {
//!     "name": "zero_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
//!     ]
//! }, {
//!     "name": "max_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
//!     ]
//! }, {
//!     "name": "ordinary_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "7437834752357434334343423343443375834785783474", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "115792089237316195423570985008680470018517627231306220616114140632078343856462"
//!     ]
//! }, {
//!     "name": "max_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "23784273472384723848213821342323233223"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "115792089237316195423570985008687907853246200392168179315609370186570806406713"
//!     ]
//! }, {
//!     "name": "max_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(uint256 a, uint256 b) external pure returns(uint256 result) {
        assembly {
            result := mul(a, b)
        }
    }
}
