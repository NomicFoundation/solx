//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f_basic()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_default()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_noname(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "7"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_multi()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_partial()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "7",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_bool()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_cond(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_cond(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "20"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_loop(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_call()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_explicit()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "99"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_int_default()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_int_neg()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "-5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_contract_default()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_enum_default()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_enum_mixed_default()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_addr_default()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_addr_set()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_bytes1_default()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_bytes1_set()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x4100000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_bytes4_default()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_bytes32_default()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_early(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_early(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_early_multi(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "10",
//!                         "20"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_early_multi(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_bytes_default()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "32",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_bytes_set()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000002",
//!                         "0x6869000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_str_default()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "32",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_str_set()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000005",
//!                         "0x68656c6c6f000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_arr_default()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "32",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_arr_set()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "32",
//!                         "2",
//!                         "1",
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_fixed_arr_default()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_fixed_arr_set()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "3",
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_int_u()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "-5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_addr_u()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_bytes1_u()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x4100000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_bytes_u()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000002",
//!                         "0x6869000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_str_u()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000005",
//!                         "0x68656c6c6f000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_arr_u()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "32",
//!                         "2",
//!                         "5",
//!                         "6"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_multi_u()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42",
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_mixed(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "7",
//!                         "6"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_mixed(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "2",
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_mixed_default()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "99"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f_arr()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2",
//!                         "3",
//!                         "4"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract CC {}

contract Test {
  uint8[33] a;
  enum Color { Red, Green, Blue }

  function f_basic() public returns (uint a) { a = 42; }

  function f_default() public returns (uint a) {}

  function f_multi() public returns (uint a, uint b) { a = 1; b = 2; }

  function f_partial() public returns (uint a, uint b) { a = 7; }

  function f_bool() public returns (bool ok) { ok = true; }

  function f_cond(bool flag) public returns (uint result) {
    if (flag) { result = 10; } else { result = 20; }
  }

  function f_loop(uint n) public returns (uint sum) {
    for (uint i = 0; i < n; i++) { sum += i; }
  }

  function f_call() public returns (uint a) { a = helper(); }
  function helper() internal returns (uint) { return 5; }

  function f_explicit() public returns (uint) { return 99; }

  function f_noname(uint b) public returns (uint) { }

  function f_int_default() public returns (int256 a) {}
  function f_int_neg() public returns (int256 a) { a = -5; }

  function f_contract_default() public returns (CC c) {}

  // Named enum return — falls off end; c must default to first enumerator (Red = 0).
  function f_enum_default() public returns (Color c) {}

  // Named enum in a mixed tuple — c defaults to Red (0), unnamed slot is explicit.
  function f_enum_mixed_default() public returns (Color c, uint) {
    return (c, 5);
  }

  function f_addr_default() public returns (address a) {}
  function f_addr_set() public returns (address a) { a = address(1); }

  function f_bytes1_default() public returns (bytes1 a) {}
  function f_bytes1_set() public returns (bytes1 a) { a = 0x41; }

  function f_bytes4_default() public returns (bytes4 a) {}

  function f_bytes32_default() public returns (bytes32 a) {}

  function f_early(bool flag) public returns (uint r) {
    r = 1;
    if (flag) return r;
    r = 2;
  }

  function f_early_multi(bool flag) public returns (uint a, uint b) {
    a = 10; b = 20;
    if (flag) return (a, b);
    a = 1; b = 2;
  }

  function f_bytes_default() public returns (bytes memory a) {}
  function f_bytes_set() public returns (bytes memory a) { a = "hi"; }

  function f_str_default() public returns (string memory a) {}
  function f_str_set() public returns (string memory a) { a = "hello"; }

  function f_arr_default() public returns (uint[] memory a) {}
  function f_arr_set() public returns (uint[] memory a) {
    a = new uint[](2);
    a[0] = 1; a[1] = 2;
  }

  function f_fixed_arr_default() public returns (uint[2] memory a) {}
  function f_fixed_arr_set() public returns (uint[2] memory a) {
    a[0] = 3; a[1] = 4;
  }

  function f_int_u() public returns (int256) { return -5; }
  function f_addr_u() public returns (address) { return address(1); }
  function f_bytes1_u() public returns (bytes1) { return 0x41; }

  function f_bytes_u() public returns (bytes memory) { return "hi"; }
  function f_str_u() public returns (string memory) { return "hello"; }
  function f_arr_u() public returns (uint[] memory) {
    uint[] memory a = new uint[](2);
    a[0] = 5; a[1] = 6;
    return a;
  }

  function f_multi_u() public returns (uint, bool) { return (42, true); }

  // Mixed named and unnamed return parameters.
  // a is named (assigned, falls off end), b is unnamed (explicit return value).
  function f_mixed(uint x) public returns (uint a, uint) {
    a = x + 2;
    return (a, x + 1);
  }

  // Mixed with default: a is named (zero default), b is unnamed (explicit).
  function f_mixed_default() public returns (uint a, uint) {
    return (a, 99);
  }

  function f_arr() public returns (uint8, uint8, uint8) {
    a[0] = 2;
    a[16] = 3;
    a[32] = 4;
    return (a[0], a[16], a[32]);
  }
}
