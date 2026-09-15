//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "call(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "6"
//!                     ]
//!                 },
//!                 {
//!                     "method": "call_options(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "8"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "9"
//!                     ]
//!                 },
//!                 {
//!                     "method": "call_fail(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "11"
//!                     ],
//!                     "expected": [
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "delegatecall(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "9"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "delegatecall_options(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "delegatecall_fail(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "staticcall(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "staticcall_options(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "7"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "8"
//!                     ]
//!                 },
//!                 {
//!                     "method": "staticcall_fail(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "10"
//!                     ],
//!                     "expected": [
//!                         "0",
//!                         "0"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  function inc(uint256 v) external pure returns (uint256) {
    return v + 1;
  }

  function call(uint256 v) external returns (bool, uint256) {
    (bool ok, bytes memory data) =
        address(this).call(abi.encodeWithSignature("inc(uint256)", v));
    if (ok == false)
      return (false, 0);
    return (true, abi.decode(data, (uint256)));
  }

  function call_options(uint256 v) external returns (bool, uint256) {
    (bool ok, bytes memory data) =
        address(this).call{gas: 100000, value: 0}(
          abi.encodeWithSignature("inc(uint256)", v)
        );
    if (ok == false)
      return (false, 0);
    return (true, abi.decode(data, (uint256)));
  }

  function call_fail(uint256 v) external returns (bool, uint256) {
    (bool ok, bytes memory data) =
        address(this).call(abi.encodeWithSignature("missing(uint256)", v));
    if (ok == false)
      return (false, 0);
    return (true, abi.decode(data, (uint256)));
  }

  function delegatecall(uint256 v) external returns (bool, uint256) {
    (bool ok, bytes memory data) =
        address(this).delegatecall(
          abi.encodeWithSignature("inc(uint256)", v)
        );
    if (ok == false)
      return (false, 0);
    return (true, abi.decode(data, (uint256)));
  }

  function delegatecall_options(uint256 v) external returns (bool, uint256) {
    (bool ok, bytes memory data) =
        address(this).delegatecall{gas: 100000}(
          abi.encodeWithSignature("inc(uint256)", v)
        );
    if (ok == false)
      return (false, 0);
    return (true, abi.decode(data, (uint256)));
  }

  function delegatecall_fail(uint256 v) external returns (bool, uint256) {
    (bool ok, bytes memory data) =
        address(this).delegatecall(
          abi.encodeWithSignature("missing(uint256)", v)
        );
    if (ok == false)
      return (false, 0);
    return (true, abi.decode(data, (uint256)));
  }

  function staticcall(uint256 v) external view returns (bool, uint256) {
    (bool ok, bytes memory data) =
        address(this).staticcall(
          abi.encodeWithSignature("inc(uint256)", v)
        );
    if (ok == false)
      return (false, 0);
    return (true, abi.decode(data, (uint256)));
  }

  function staticcall_options(uint256 v) external view returns (bool, uint256) {
    (bool ok, bytes memory data) =
        address(this).staticcall{gas: 100000}(
          abi.encodeWithSignature("inc(uint256)", v)
        );
    if (ok == false)
      return (false, 0);
    return (true, abi.decode(data, (uint256)));
  }

  function staticcall_fail(uint256 v) external view returns (bool, uint256) {
    (bool ok, bytes memory data) =
        address(this).staticcall(
          abi.encodeWithSignature("missing(uint256)", v)
        );
    if (ok == false)
      return (false, 0);
    return (true, abi.decode(data, (uint256)));
  }
}
