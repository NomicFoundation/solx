//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "static_arr_rw()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dynamic_arr_pop()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000003100000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "dynamic_arr_rw(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000003200000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "dynamic_arr_rw(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dynamic_arr_pop()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "static_2d_rw()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "12"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dynamic_2d_rw()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "2",
//!                         "0x10",
//!                         "0x20",
//!                         "0x30",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "return_as_mem()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "2",
//!                         "1",
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_packed_static(uint256,uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "0x11"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "set_packed_static(uint256,uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0x22"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "set_packed_static(uint256,uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2",
//!                         "0x33"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "set_packed_static(uint256,uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "3",
//!                         "0x44"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_packed_static(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "0x11"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_packed_static(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "0x22"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_packed_static(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2"
//!                     ],
//!                     "expected": [
//!                         "0x33"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_packed_static(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "3"
//!                     ],
//!                     "expected": [
//!                         "0x44"
//!                     ]
//!                 },
//!                 {
//!                     "method": "len_packed_dynamic()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "push_packed_dynamic(uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x11"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "push_packed_dynamic(uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x22"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "push_packed_dynamic(uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x33"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "push_packed_dynamic(uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x44"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "len_packed_dynamic()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_packed_dynamic(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "0x11"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_packed_dynamic(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "0x22"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_packed_dynamic(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2"
//!                     ],
//!                     "expected": [
//!                         "0x33"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_packed_dynamic(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "3"
//!                     ],
//!                     "expected": [
//!                         "0x44"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  // Static uint array
  uint[5] static_arr;
  function static_arr_rw() public returns (uint) {
    static_arr[0] = 1;
    return static_arr[0];
  }

  // Dynamic uint array
  uint[] dynamic_arr;
  function dynamic_arr_rw(uint a) public returns (uint) {
    if (a != 0) {
      dynamic_arr.push();
      dynamic_arr.push() = a;
      dynamic_arr.push(1 + dynamic_arr[0] + dynamic_arr[1]);
    }
    return dynamic_arr[2];
  }

  function dynamic_arr_pop() public returns (uint) {
    dynamic_arr.pop();
    return 0;
  }

  // 2D static array
  uint[3][2] static_2d;
  function static_2d_rw() public returns (uint) {
  unchecked {
    for (uint i = 0; i < 2; ++i)
      for (uint j = 0; j < 3; ++j)
        static_2d[i][j] = i*10 + j;
    return static_2d[1][2];
  }
  }

  // Dynamic array of static arrays
  uint[2][] dynamic_2d;
  function dynamic_2d_rw() public returns (uint[2][] memory) {
    dynamic_2d.push()[0] = 0x10;
    dynamic_2d[0][1] = 0x20;
    dynamic_2d.push();
    dynamic_2d[1][0] = 0x30;
    return dynamic_2d;
  }

  // Return storage array as memory
  uint[] m;
  function return_as_mem() public returns (uint[] memory) {
    m.push() = 1;
    m.push(2);
    return m;
  }

  // Packed static array
  uint8[4] packed_static;
  function set_packed_static(uint256 i, uint8 v) public {
    packed_static[i] = v;
  }
  function get_packed_static(uint256 i) public view returns (uint8) {
    return packed_static[i];
  }

  // Packed dynamic array
  uint8[] packed_dynamic;
  function push_packed_dynamic(uint8 v) public {
    packed_dynamic.push(v);
  }
  function get_packed_dynamic(uint256 i) public view returns (uint8) {
    return packed_dynamic[i];
  }
  function len_packed_dynamic() public view returns (uint256) {
    return packed_dynamic.length;
  }
}
