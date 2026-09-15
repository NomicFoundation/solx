//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "deploy()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "call(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "41"
//!                     ],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tryCall(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "41"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "42"
//!                     ]
//!                 },
//!                 {
//!                     "method": "callThroughReturned()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 },
//!                 {
//!                     "method": "storeAndLoad()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "7"
//!                     ]
//!                 },
//!                 {
//!                     "method": "packedStorage()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "11",
//!                         "99"
//!                     ]
//!                 },
//!                 {
//!                     "method": "packedOverwriteVal()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "11",
//!                         "77"
//!                     ]
//!                 },
//!                 {
//!                     "method": "memoryArray()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "33"
//!                     ]
//!                 },
//!                 {
//!                     "method": "memoryStruct()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "12"
//!                     ]
//!                 },
//!                 {
//!                     "method": "encodePackedSingle()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "24"
//!                     ]
//!                 },
//!                 {
//!                     "method": "encodePackedArray()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "64",
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "abiRoundtripSingle()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "7"
//!                     ]
//!                 },
//!                 {
//!                     "method": "abiRoundtripArray()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "packedOverwriteFn()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "17",
//!                         "77"
//!                     ]
//!                 },
//!                 {
//!                     "method": "acceptFn(function)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000123412345678000000000000000000000001"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "encodeStorageArray()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "*"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

interface I {
  function h(uint x) external pure returns (uint);
}

contract D is I {
  function h(uint x) external pure returns (uint) { return x + 1; }
  function add(uint a, uint b) external pure returns (uint) { return a + b; }
}

struct FnStruct {
  function(uint, uint) external pure returns (uint) fn;
  uint x;
}

contract Test {
  I t;

  function(uint, uint) external pure returns (uint) storedAdd;
  uint64 public packedVal;

  function deploy() public { t = new D(); }

  // Call through external function pointer.
  function call(uint x) public view returns (uint) {
    function(uint) external pure returns (uint) fp = t.h;
    return fp(x);
  }

  // Try-call through external function pointer.
  function tryCall(uint x) public returns (bool, uint) {
    function(uint) external pure returns (uint) fp = t.h;
    bool success;
    uint result;
    try fp(x) returns (uint r) {
      success = true;
      result = r;
    } catch {
      success = false;
      result = 0;
    }
    return (success, result);
  }

  // Return external function pointer, then call through it.
  function callThroughReturned() public returns (uint) {
    function(uint, uint) external pure returns (uint) fn = this.getAddFn();
    return fn(20, 22);
  }

  function getAddFn()
    public view returns (function(uint, uint) external pure returns (uint))
  {
    return D(address(t)).add;
  }

  // Storage: store, load, call.
  function storeAndLoad() public returns (uint) {
    storedAdd = D(address(t)).add;
    return storedAdd(3, 4);
  }

  // Packed storage: external function pointer (24B) + uint64 (8B) = 1 slot.
  function packedStorage() public returns (uint, uint64) {
    storedAdd = D(address(t)).add;
    packedVal = 99;
    return (storedAdd(5, 6), packedVal);
  }

  // Packed storage: overwrite only uint64, function pointer survives.
  function packedOverwriteVal() public returns (uint, uint64) {
    packedVal = 77;
    return (storedAdd(5, 6), packedVal);
  }

  // Memory array of external function pointers.
  function memoryArray() public returns (uint) {
    function(uint, uint) external pure returns (uint)[] memory fns =
      new function(uint, uint) external pure returns (uint)[](2);
    fns[0] = D(address(t)).add;
    fns[1] = D(address(t)).add;
    function(uint, uint) external pure returns (uint) fn0 = fns[0];
    function(uint, uint) external pure returns (uint) fn1 = fns[1];
    return fn0(10, 20) + fn1(1, 2);
  }

  // External function pointer as memory struct field.
  function memoryStruct() public returns (uint) {
    FnStruct memory s;
    s.fn = D(address(t)).add;
    s.x = 5;
    function(uint, uint) external pure returns (uint) fn = s.fn;
    uint x = s.x;
    return fn(x, 7);
  }

  // abi.encodePacked on a single external function pointer: 24 bytes.
  function encodePackedSingle() public view returns (uint) {
    function(uint, uint) external pure returns (uint) fp = D(address(t)).add;
    bytes memory packed = abi.encodePacked(fp);
    return packed.length;
  }

  // abi.encodePacked on an array of external function pointers.
  // Arrays encode each element padded to 32 bytes.
  function encodePackedArray() public view returns (uint, bool) {
    function(uint, uint) external pure returns (uint)[] memory fns =
      new function(uint, uint) external pure returns (uint)[](2);
    fns[0] = D(address(t)).add;
    fns[1] = D(address(t)).add;
    bytes memory packed = abi.encodePacked(fns);
    // Both elements are identical, so first 32 bytes == last 32 bytes.
    bytes32 first;
    bytes32 second;
    assembly {
      first := mload(add(packed, 32))
      second := mload(add(packed, 64))
    }
    return (packed.length, first == second);
  }

  // abi.encode/decode roundtrip of a plain external function pointer.
  // Uses this.helper() to force ABI encode on call, decode on receive.
  function abiRoundtripSingle() public returns (uint) {
    function(uint, uint) external pure returns (uint) fp = D(address(t)).add;
    return this.abiRoundtripSingleHelper(fp);
  }
  function abiRoundtripSingleHelper(
    function(uint, uint) external pure returns (uint) fp
  ) external pure returns (uint) {
    return fp(3, 4);
  }

  // abi.encode/decode roundtrip with ext fn ptr inside an array.
  function abiRoundtripArray() public returns (uint) {
    function(uint, uint) external pure returns (uint)[] memory fns =
      new function(uint, uint) external pure returns (uint)[](2);
    fns[0] = D(address(t)).add;
    fns[1] = D(address(t)).add;
    return this.abiRoundtripArrayHelper(fns);
  }
  function abiRoundtripArrayHelper(
    function(uint, uint) external pure returns (uint)[] calldata fns
  ) external pure returns (uint) {
    return fns[0](1, 2) + fns[1](3, 4);
  }

  // Packed storage: overwrite fn ptr, uint64 survives.
  function packedOverwriteFn() public returns (uint, uint64) {
    storedAdd = D(address(t)).add;
    return (storedAdd(8, 9), packedVal);
  }

  // Accepts a single ext fn ptr (for dirty-decode test via raw calldata).
  function acceptFn(
    function(uint) external pure returns (uint) fp
  ) external pure returns (uint) {
    return 1;
  }

  function() external[] fns;

  function fn() external {}

  // ABI-encode directly from storage (raw sload of the storage-form word).
  function encodeStorageArray() public returns (bytes32) {
    fns.push(this.fn);
    return keccak256(abi.encode(fns));
  }
}
