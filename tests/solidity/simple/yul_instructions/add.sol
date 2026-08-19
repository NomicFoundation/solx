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
//!         "1"
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
//!         "1"
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
//!         "2"
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
//!         "433478394034343"
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
//!         "121563127839120"
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
//!         "433478394034344"
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
//!         "121563127839121"
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
//!         "3907956900864202467"
//!     ]
//! }, {
//!     "name": "ordinary_ordinary_to_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xfafffffffffffffffffffffffffffffffffffffffffffffffffffffff4ffffff", "0x050000000000000000000000000000000000000000000000000000000b000000"
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
//!                 "0xab000000000000000000000000000000000000000000000000000000000000a1", "0x54ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff5f"
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
//!                 "0x2fff1ffffffffff5ffffff0fffffffff2ffffffafffafffcffff1ff234ffffff", "0xef231ffffffffff4f12fff34ffffffff2fffffbbfffafffdffff22f538ffffff"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x1f223fffffffffeaf12ffe44fffffffe5fffffb6fff5fffafffe42e76dfffffe"
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
//!         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
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
//!         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
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
//!         "0"
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
//!         "0"
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
//!         "7437834752357434334343423343443375834785783473"
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
//!         "23784273472384723848213821342323233222"
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
//!         "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(uint256 a, uint256 b) external pure returns(uint256 result) {
        assembly {
            result := add(a, b)
        }
    }
}
