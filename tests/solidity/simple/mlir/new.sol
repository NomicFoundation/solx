//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "g()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 }
//!             ]
//!         },
//!         {
//!             "name": "inherited_function",
//!             "inputs": [
//!                 {
//!                     "method": "inherited()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 }
//!             ]
//!         },
//!         {
//!             "name": "base_constructor",
//!             "inputs": [
//!                 {
//!                     "method": "fromBaseConstructor()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 }
//!             ]
//!         },
//!         {
//!             "name": "base_argument",
//!             "inputs": [
//!                 {
//!                     "method": "fromBaseArgument()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 }
//!             ]
//!         },
//!         {
//!             "name": "constructor_stored_pointer",
//!             "inputs": [
//!                 {
//!                     "method": "storedPointer()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract C {
  function f() public returns (uint) { return 42; }
}

contract D is C {}

contract E is C {}

contract F is C {}

contract G is C {}

abstract contract Inherited {
  function inherited() public returns (uint) { return new D().f(); }
}

abstract contract BaseConstructor {
  uint public fromBaseConstructor;

  constructor() { fromBaseConstructor = new E().f(); }
}

abstract contract BaseArgument {
  uint public fromBaseArgument;

  constructor(uint value) { fromBaseArgument = value; }
}

contract Test is Inherited, BaseConstructor, BaseArgument(new F().f()) {
  function() internal returns (uint) stored;

  constructor() { stored = created; }

  function created() internal returns (uint) { return new G().f(); }

  function g() public returns (uint) { return new C().f(); }

  function storedPointer() public returns (uint) { return stored(); }
}
