//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "declaration()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "assignment()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "argument()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "array_push()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "bytes_push()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x6100000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "comparison()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "conversion()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "mapping_index()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "7"
//!                     ]
//!                 },
//!                 {
//!                     "method": "event_argument()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [],
//!                         "events": [
//!                             {
//!                                 "topics": [
//!                                     "0xd3e1c28fce70026151246c05dc1ec62eca91daf00dad6d81ae7f0c409bc99cb1",
//!                                     "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                                 ],
//!                                 "values": []
//!                             }
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "struct_field()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "hex_literal()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0xabcd000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "require_message()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x08c379a000000000000000000000000000000000000000000000000000000000",
//!                             "0x0000002000000000000000000000000000000000000000000000000000000000",
//!                             "0x000000036d736700000000000000000000000000000000000000000000000000",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "revert_message()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x08c379a000000000000000000000000000000000000000000000000000000000",
//!                             "0x0000002000000000000000000000000000000000000000000000000000000000",
//!                             "0x000000036d736700000000000000000000000000000000000000000000000000",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "encode_with_signature()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000004",
//!                         "0x26121ff000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  event Ev(bytes32 indexed a);

  struct S {
    bytes32 a;
  }

  bytes32 stored;
  bytes32[] array;
  bytes data;
  mapping(bytes32 => uint256) map;

  function identity(bytes32 x) internal pure returns (bytes32) {
    return x;
  }

  function declaration() public pure returns (bytes32) {
    bytes32 x = ("abc");
    return x;
  }

  function assignment() public returns (bytes32) {
    stored = ("abc");
    return stored;
  }

  function argument() public pure returns (bytes32) {
    return identity(("abc"));
  }

  function array_push() public returns (bytes32) {
    array.push(("abc"));
    return array[array.length - 1];
  }

  function bytes_push() public returns (bytes1) {
    data.push(("a"));
    return data[data.length - 1];
  }

  function comparison() public pure returns (bool) {
    bytes32 x = "abc";
    return x == ("abc");
  }

  function conversion() public pure returns (bytes32) {
    return bytes32(("abc"));
  }

  function mapping_index() public returns (uint256) {
    map[("abc")] = 7;
    return map["abc"];
  }

  function event_argument() public {
    emit Ev(("abc"));
  }

  function struct_field() public pure returns (bytes32) {
    S memory value = S(("abc"));
    return value.a;
  }

  function hex_literal() public pure returns (bytes4) {
    bytes4 x = (hex"abcd");
    return x;
  }

  function require_message() public pure {
    require(false, ("msg"));
  }

  function revert_message() public pure {
    revert(("msg"));
  }

  function encode_with_signature() public pure returns (bytes memory) {
    return abi.encodeWithSignature(("f()"));
  }
}
