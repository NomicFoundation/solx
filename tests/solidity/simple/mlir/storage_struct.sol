//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "set_packed(uint8,uint8,uint8,uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x11",
//!                         "0x22",
//!                         "0x33",
//!                         "0x44"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_packed()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x11",
//!                         "0x22",
//!                         "0x33",
//!                         "0x44"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_non_packed(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0xaa",
//!                         "0xbb"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_non_packed()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0xaa",
//!                         "0xbb"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_mixed(uint8,uint256,uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x11",
//!                         "0xaa",
//!                         "0x22"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_mixed()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x11",
//!                         "0xaa",
//!                         "0x22"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_nested(uint8,uint8,uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x11",
//!                         "0x22",
//!                         "0x33"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_nested()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x11",
//!                         "0x22",
//!                         "0x33"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_mixed_ref(uint8,uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0xaa",
//!                         "0xbb"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_mixed_ref()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0xaa",
//!                         "0",
//!                         "0xbb"
//!                     ]
//!                 },
//!                 {
//!                     "method": "push_mixed_ref(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x111"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "push_mixed_ref(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x222"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_mixed_ref()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0xaa",
//!                         "2",
//!                         "0xbb"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_mixed_ref_arr(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "0x111"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_mixed_ref_arr(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "0x222"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_contract_ref(uint96,address)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "7",
//!                         "0x1212121212121212121212121212121212121212"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_contract_ref()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "7",
//!                         "0x1212121212121212121212121212121212121212"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_nested_dynamic(uint256,uint256,uint128,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "100",
//!                         "201",
//!                         "202",
//!                         "300"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_nested_dynamic()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000064",
//!                         "0x00000000000000000000000000000000000000000000000000000000000000c9",
//!                         "0x00000000000000000000000000000000000000000000000000000000000000ca",
//!                         "0x00000000000000000000000000000000000000000000000000000000000000a0",
//!                         "0x000000000000000000000000000000000000000000000000000000000000012c",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000002",
//!                         "0x696e000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_fixed_array_tail(uint8,uint8,uint8,uint8,uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4",
//!                         "0xaaaa",
//!                         "0xbbbb"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_fixed_array_tail()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4",
//!                         "0xaaaa",
//!                         "0xbbbb"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  // Packed struct (all fields in one slot)
  struct Packed {
    uint8 a;
    uint8 b;
    uint8 c;
    uint8 d;
  }
  Packed packed;
  function set_packed(uint8 a, uint8 b, uint8 c, uint8 d) public {
    packed.a = a;
    packed.b = b;
    packed.c = c;
    packed.d = d;
  }
  function get_packed() public view returns (uint8, uint8, uint8, uint8) {
    return (packed.a, packed.b, packed.c, packed.d);
  }

  // Non-packed struct (fields span multiple slots)
  struct NonPacked {
    uint256 x;
    uint256 y;
  }
  NonPacked non_packed;
  function set_non_packed(uint256 x, uint256 y) public {
    non_packed.x = x;
    non_packed.y = y;
  }
  function get_non_packed() public view returns (uint256, uint256) {
    return (non_packed.x, non_packed.y);
  }

  // Mixed struct (packed and non-packed fields)
  struct Mixed {
    uint8 a;
    uint256 b;
    uint8 c;
  }
  Mixed mixed;
  function set_mixed(uint8 a, uint256 b, uint8 c) public {
    mixed.a = a;
    mixed.b = b;
    mixed.c = c;
  }
  function get_mixed() public view returns (uint8, uint256, uint8) {
    return (mixed.a, mixed.b, mixed.c);
  }

  // Nested struct
  struct Inner {
    uint8 x;
    uint8 y;
  }
  struct Outer {
    Inner inner;
    uint8 z;
  }
  Outer nested;
  function set_nested(uint8 x, uint8 y, uint8 z) public {
    nested.inner.x = x;
    nested.inner.y = y;
    nested.z = z;
  }
  function get_nested() public view returns (uint8, uint8, uint8) {
    return (nested.inner.x, nested.inner.y, nested.z);
  }

  // Struct with value + reference types
  struct MixedRef {
    uint8 x;
    uint256[] arr;
    uint8 y;
  }
  MixedRef mixed_ref;
  function set_mixed_ref(uint8 x, uint8 y) public {
    mixed_ref.x = x;
    mixed_ref.y = y;
  }
  function push_mixed_ref(uint256 val) public {
    mixed_ref.arr.push(val);
  }
  function get_mixed_ref() public view returns (uint8, uint256, uint8) {
    return (mixed_ref.x, mixed_ref.arr.length, mixed_ref.y);
  }
  function get_mixed_ref_arr(uint256 idx) public view returns (uint256) {
    return mixed_ref.arr[idx];
  }

  struct ContractRef {
    uint96 x;
    Test c;
  }
  ContractRef contract_ref;
  function set_contract_ref(uint96 x, Test c) public {
    contract_ref.x = x;
    contract_ref.c = c;
  }
  function get_contract_ref() public view returns (uint96, Test) {
    return (contract_ref.x, contract_ref.c);
  }

  // Nested struct where inner has a dynamic member.
  struct InnerDynamic {
    uint256 a;
    uint128 b;
    string note;
  }
  struct OuterDynamic {
    uint256 left;
    InnerDynamic inner;
    uint256 right;
  }
  OuterDynamic nested_dynamic;
  function set_nested_dynamic(uint256 left, uint256 a, uint128 b, uint256 right) public {
    nested_dynamic.left = left;
    nested_dynamic.inner.a = a;
    nested_dynamic.inner.b = b;
    nested_dynamic.inner.note = "in";
    nested_dynamic.right = right;
  }
  function get_nested_dynamic() public view returns (uint256, uint256, uint128, string memory, uint256) {
    return (nested_dynamic.left, nested_dynamic.inner.a, nested_dynamic.inner.b, nested_dynamic.inner.note, nested_dynamic.right);
  }

  // Packed fixed-array member followed by scalar member.
  struct FixedArrayTail {
    uint8[4] a;
    uint256 tail;
  }
  FixedArrayTail fixed_array_tail;
  uint256 fixed_array_tail_pad0;
  uint256 fixed_array_tail_pad1;
  uint256 fixed_array_tail_guard;
  function set_fixed_array_tail(uint8 a0, uint8 a1, uint8 a2, uint8 a3, uint256 tail, uint256 guard) public {
    fixed_array_tail.a[0] = a0;
    fixed_array_tail.a[1] = a1;
    fixed_array_tail.a[2] = a2;
    fixed_array_tail.a[3] = a3;
    fixed_array_tail.tail = tail;
    fixed_array_tail_guard = guard;
  }
  function get_fixed_array_tail() public view returns (uint8, uint8, uint8, uint8, uint256, uint256) {
    return (fixed_array_tail.a[0], fixed_array_tail.a[1], fixed_array_tail.a[2], fixed_array_tail.a[3], fixed_array_tail.tail, fixed_array_tail_guard);
  }
}
