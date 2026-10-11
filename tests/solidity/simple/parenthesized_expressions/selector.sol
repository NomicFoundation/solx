//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "instance_function()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0be5edb600000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "interface_function()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0xb3de648b00000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "base_function()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x7afa697b00000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "bare_function()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x7afa697b00000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "side_effect()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 },
//!                 {
//!                     "method": "encode_call_interface()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000024",
//!                         "0xb3de648b00000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000100000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "encode_call_instance()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000024",
//!                         "0x0be5edb600000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000100000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

interface I {
  function f(uint256 x) external returns (uint256);
}

contract B {
  function bp(uint256 x) public pure returns (uint256) {
    return x;
  }
}

contract Test is B {
  uint256 counter;

  function ext(uint256 x) external pure returns (uint256) {
    return x;
  }

  function get() public returns (Test) {
    counter = 42;
    return this;
  }

  function instance_function() public view returns (bytes4) {
    return (this.ext).selector;
  }

  function interface_function() public pure returns (bytes4) {
    return (I.f).selector;
  }

  function base_function() public pure returns (bytes4) {
    return (B.bp).selector;
  }

  function bare_function() public pure returns (bytes4) {
    return bp.selector;
  }

  function side_effect() public returns (uint256) {
    (get().ext).selector;
    return counter;
  }

  function encode_call_interface() public pure returns (bytes memory) {
    return abi.encodeCall((I.f), (1));
  }

  function encode_call_instance() public view returns (bytes memory) {
    return abi.encodeCall((this.ext), (1));
  }
}
