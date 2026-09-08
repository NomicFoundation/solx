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
//!                     "expected": [
//!                         "Test.address"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ei_u256_array_dynamic_storage()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "160",
//!                         "0x20",
//!                         "3",
//!                         "11",
//!                         "22",
//!                         "33"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ei_u8_array_static_storage_4()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "128",
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ei_u8_array_static_storage_31()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "992",
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4",
//!                         "5",
//!                         "6",
//!                         "7",
//!                         "8",
//!                         "9",
//!                         "10",
//!                         "11",
//!                         "12",
//!                         "13",
//!                         "14",
//!                         "15",
//!                         "16",
//!                         "17",
//!                         "18",
//!                         "19",
//!                         "20",
//!                         "21",
//!                         "22",
//!                         "23",
//!                         "24",
//!                         "25",
//!                         "26",
//!                         "27",
//!                         "28",
//!                         "29",
//!                         "30",
//!                         "31"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ei_u8_array_static_storage_32()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "1024",
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4",
//!                         "5",
//!                         "6",
//!                         "7",
//!                         "8",
//!                         "9",
//!                         "10",
//!                         "11",
//!                         "12",
//!                         "13",
//!                         "14",
//!                         "15",
//!                         "16",
//!                         "17",
//!                         "18",
//!                         "19",
//!                         "20",
//!                         "21",
//!                         "22",
//!                         "23",
//!                         "24",
//!                         "25",
//!                         "26",
//!                         "27",
//!                         "28",
//!                         "29",
//!                         "30",
//!                         "31",
//!                         "32"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ei_u8_array_static_storage_33()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "1056",
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4",
//!                         "5",
//!                         "6",
//!                         "7",
//!                         "8",
//!                         "9",
//!                         "10",
//!                         "11",
//!                         "12",
//!                         "13",
//!                         "14",
//!                         "15",
//!                         "16",
//!                         "17",
//!                         "18",
//!                         "19",
//!                         "20",
//!                         "21",
//!                         "22",
//!                         "23",
//!                         "24",
//!                         "25",
//!                         "26",
//!                         "27",
//!                         "28",
//!                         "29",
//!                         "30",
//!                         "31",
//!                         "32",
//!                         "33"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ei_u256_array_static_storage_3()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "96",
//!                         "1",
//!                         "2",
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ei_u256_array_array_dynamic_storage()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "288",
//!                         "0x20",
//!                         "2",
//!                         "0x40",
//!                         "0xa0",
//!                         "2",
//!                         "101",
//!                         "202",
//!                         "1",
//!                         "303"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ei_u256x2_array_dynamic_storage()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "192",
//!                         "0x20",
//!                         "2",
//!                         "10",
//!                         "20",
//!                         "30",
//!                         "40"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ei_u256_array_static_storage_2()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "256",
//!                         "0x20",
//!                         "0x40",
//!                         "0xa0",
//!                         "2",
//!                         "7",
//!                         "8",
//!                         "1",
//!                         "9"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ei_u256_pair_array_dynamic_storage()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "192",
//!                         "0x20",
//!                         "2",
//!                         "111",
//!                         "222",
//!                         "333",
//!                         "444"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ei_u256_pair_array_static_storage_2()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "128",
//!                         "555",
//!                         "666",
//!                         "777",
//!                         "888"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  struct U256Pair {
    uint256 left;
    uint256 right;
  }

  uint256[] dyn256;

  uint8[4] static8x4;
  uint8[31] static8x31;
  uint8[32] static8x32;
  uint8[33] static8x33;
  uint256[3] static256x3;

  uint256[][] dynRefDyn256;
  uint256[2][] dynRefStatic2x;
  uint256[][2] staticRefDyn256x2;
  U256Pair[] dynStructPair;
  U256Pair[2] staticStructPair2;

  constructor() {
    dyn256.push(11);
    dyn256.push(22);
    dyn256.push(33);

    static8x4[0] = 1;
    static8x4[1] = 2;
    static8x4[2] = 3;
    static8x4[3] = 4;

    for (uint256 i = 0; i < 31; ++i)
      static8x31[i] = uint8(i + 1);

    for (uint256 i = 0; i < 32; ++i)
      static8x32[i] = uint8(i + 1);

    for (uint256 i = 0; i < 33; ++i)
      static8x33[i] = uint8(i + 1);

    static256x3[0] = 1;
    static256x3[1] = 2;
    static256x3[2] = 3;

    dynRefDyn256.push();
    dynRefDyn256[0].push(101);
    dynRefDyn256[0].push(202);
    dynRefDyn256.push();
    dynRefDyn256[1].push(303);

    dynRefStatic2x.push();
    dynRefStatic2x[0][0] = 10;
    dynRefStatic2x[0][1] = 20;
    dynRefStatic2x.push();
    dynRefStatic2x[1][0] = 30;
    dynRefStatic2x[1][1] = 40;

    staticRefDyn256x2[0].push(7);
    staticRefDyn256x2[0].push(8);
    staticRefDyn256x2[1].push(9);

    dynStructPair.push();
    dynStructPair[0].left = 111;
    dynStructPair[0].right = 222;
    dynStructPair.push();
    dynStructPair[1].left = 333;
    dynStructPair[1].right = 444;

    staticStructPair2[0].left = 555;
    staticStructPair2[0].right = 666;
    staticStructPair2[1].left = 777;
    staticStructPair2[1].right = 888;
  }

  function ei_u256_array_dynamic_storage() public view returns (bytes memory) {
    return abi.encode(dyn256);
  }

  function ei_u8_array_static_storage_4() public view returns (bytes memory) {
    return abi.encode(static8x4);
  }

  function ei_u8_array_static_storage_31() public view returns (bytes memory) {
    return abi.encode(static8x31);
  }

  function ei_u8_array_static_storage_32() public view returns (bytes memory) {
    return abi.encode(static8x32);
  }

  function ei_u8_array_static_storage_33() public view returns (bytes memory) {
    return abi.encode(static8x33);
  }

  function ei_u256_array_static_storage_3() public view returns (bytes memory) {
    return abi.encode(static256x3);
  }

  function ei_u256_array_array_dynamic_storage() public view returns (bytes memory) {
    return abi.encode(dynRefDyn256);
  }

  function ei_u256x2_array_dynamic_storage() public view returns (bytes memory) {
    return abi.encode(dynRefStatic2x);
  }

  function ei_u256_array_static_storage_2() public view returns (bytes memory) {
    return abi.encode(staticRefDyn256x2);
  }

  function ei_u256_pair_array_dynamic_storage() public view returns (bytes memory) {
    return abi.encode(dynStructPair);
  }

  function ei_u256_pair_array_static_storage_2() public view returns (bytes memory) {
    return abi.encode(staticStructPair2);
  }
}
