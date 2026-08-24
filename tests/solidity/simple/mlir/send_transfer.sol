//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "#deployer",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "value": "10 wei",
//!                     "expected": [
//!                         "Test.address"
//!                     ]
//!                 },
//!                 {
//!                     "method": "sendOk()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "sendFail()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "transferOk()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "transferFail()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Receiver {
  receive() external payable {}
}

contract Rejector {
  receive() external payable {
    revert();
  }
}

contract Test {
  constructor() payable {}

  function sendOk() public returns (bool) {
    Receiver r = new Receiver();
    return payable(r).send(1 wei);
  }

  function sendFail() public returns (bool) {
    Rejector r = new Rejector();
    return payable(r).send(1 wei);
  }

  function transferOk() public {
    Receiver r = new Receiver();
    payable(r).transfer(1 wei);
  }

  function transferFail() public {
    Rejector r = new Rejector();
    payable(r).transfer(1 wei);
  }
}
