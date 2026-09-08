//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "tail_after_encode_bytes_memory(bytes)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tail_after_encode_bytes_calldata(bytes)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tail_after_encode_string_memory(string)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tail_after_encode_string_calldata(string)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tail_after_packed_bytes_memory(bytes)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tail_after_packed_bytes_calldata(bytes)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tail_after_packed_string_memory(string)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tail_after_packed_string_calldata(string)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0x6162630000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
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
  function tail_after_encode_bytes_memory(bytes memory x) public pure returns (uint256) {
    bytes memory out;

    assembly {
      // Force abi.encode allocation to reuse dirty memory.
      let p := mload(0x40)
      mstore(add(p, 0x60), not(0))
      mstore(add(p, 0x80), not(0))
      mstore(0x40, p)
    }

    out = abi.encode(x);
    uint256 tail;
    assembly {
      let payload := add(out, 0x20)
      let len := mload(add(payload, 0x20))
      tail := mload(add(add(payload, 0x40), len))
    }
    return tail;
  }

  function tail_after_encode_bytes_calldata(bytes calldata x) external pure returns (uint256) {
    bytes memory out;

    assembly {
      // Force abi.encode allocation to reuse dirty memory.
      let p := mload(0x40)
      mstore(add(p, 0x60), not(0))
      mstore(add(p, 0x80), not(0))
      mstore(0x40, p)
    }

    out = abi.encode(x);
    uint256 tail;
    assembly {
      let payload := add(out, 0x20)
      let len := mload(add(payload, 0x20))
      tail := mload(add(add(payload, 0x40), len))
    }
    return tail;
  }

  function tail_after_encode_string_memory(string memory x) public pure returns (uint256) {
    bytes memory out;

    assembly {
      // Force abi.encode allocation to reuse dirty memory.
      let p := mload(0x40)
      mstore(add(p, 0x60), not(0))
      mstore(add(p, 0x80), not(0))
      mstore(0x40, p)
    }

    out = abi.encode(x);
    uint256 tail;
    assembly {
      let payload := add(out, 0x20)
      let len := mload(add(payload, 0x20))
      tail := mload(add(add(payload, 0x40), len))
    }
    return tail;
  }

  function tail_after_encode_string_calldata(string calldata x) external pure returns (uint256) {
    bytes memory out;

    assembly {
      // Force abi.encode allocation to reuse dirty memory.
      let p := mload(0x40)
      mstore(add(p, 0x60), not(0))
      mstore(add(p, 0x80), not(0))
      mstore(0x40, p)
    }

    out = abi.encode(x);
    uint256 tail;
    assembly {
      let payload := add(out, 0x20)
      let len := mload(add(payload, 0x20))
      tail := mload(add(add(payload, 0x40), len))
    }
    return tail;
  }

  function tail_after_packed_bytes_memory(bytes memory x) public pure returns (uint256) {
    bytes memory out;

    assembly {
      // Force abi.encodePacked allocation to reuse dirty memory.
      let p := mload(0x40)
      mstore(add(p, 0x20), not(0))
      mstore(add(p, 0x40), not(0))
      mstore(0x40, p)
    }

    out = abi.encodePacked(x);
    uint256 tail;
    assembly {
      tail := mload(add(add(out, 0x20), mload(out)))
    }
    return tail;
  }

  function tail_after_packed_bytes_calldata(bytes calldata x) external pure returns (uint256) {
    bytes memory out;

    assembly {
      // Force abi.encodePacked allocation to reuse dirty memory.
      let p := mload(0x40)
      mstore(add(p, 0x20), not(0))
      mstore(add(p, 0x40), not(0))
      mstore(0x40, p)
    }

    out = abi.encodePacked(x);
    uint256 tail;
    assembly {
      tail := mload(add(add(out, 0x20), mload(out)))
    }
    return tail;
  }

  function tail_after_packed_string_memory(string memory x) public pure returns (uint256) {
    bytes memory out;

    assembly {
      // Force abi.encodePacked allocation to reuse dirty memory.
      let p := mload(0x40)
      mstore(add(p, 0x20), not(0))
      mstore(add(p, 0x40), not(0))
      mstore(0x40, p)
    }

    out = abi.encodePacked(x);
    uint256 tail;
    assembly {
      tail := mload(add(add(out, 0x20), mload(out)))
    }
    return tail;
  }

  function tail_after_packed_string_calldata(string calldata x) external pure returns (uint256) {
    bytes memory out;

    assembly {
      // Force abi.encodePacked allocation to reuse dirty memory.
      let p := mload(0x40)
      mstore(add(p, 0x20), not(0))
      mstore(add(p, 0x40), not(0))
      mstore(0x40, p)
    }

    out = abi.encodePacked(x);
    uint256 tail;
    assembly {
      tail := mload(add(add(out, 0x20), mload(out)))
    }
    return tail;
  }
}
