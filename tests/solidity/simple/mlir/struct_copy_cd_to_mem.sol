//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "i72SlotRaw((int72,uint8,bool[][],address[2],int192,uint8,bytes4,int8,int16,int128,uint72,uint128,uint224,bytes8,bytes19))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000200",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
//!                     ]
//!                 },
//!                 {
//!                     "method": "i192SlotRaw((int72,uint8,bool[][],address[2],int192,uint8,bytes4,int8,int16,int128,uint72,uint128,uint224,bytes8,bytes19))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000200",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
//!                     ]
//!                 },
//!                 {
//!                     "method": "i8SlotRaw((int72,uint8,bool[][],address[2],int192,uint8,bytes4,int8,int16,int128,uint72,uint128,uint224,bytes8,bytes19))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000200",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80"
//!                     ]
//!                 },
//!                 {
//!                     "method": "i16SlotRaw((int72,uint8,bool[][],address[2],int192,uint8,bytes4,int8,int16,int128,uint72,uint128,uint224,bytes8,bytes19))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000200",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300"
//!                     ]
//!                 },
//!                 {
//!                     "method": "b4SlotRaw((int72,uint8,bool[][],address[2],int192,uint8,bytes4,int8,int16,int128,uint72,uint128,uint224,bytes8,bytes19))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000200",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "b8SlotRaw((int72,uint8,bool[][],address[2],int192,uint8,bytes4,int8,int16,int128,uint72,uint128,uint224,bytes8,bytes19))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000200",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "b19SlotRaw((int72,uint8,bool[][],address[2],int192,uint8,bytes4,int8,int16,int128,uint72,uint128,uint224,bytes8,bytes19))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000200",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "cdToMem((int72,uint8,bool[][],address[2],int192,uint8,bytes4,int8,int16,int128,uint72,uint128,uint224,bytes8,bytes19))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000200",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "cdToMemExt((int72,uint8,bool[][],address[2],int192,uint8,bytes4,int8,int16,int128,uint72,uint128,uint224,bytes8,bytes19))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000200",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "i72SlotRaw((int72,uint8,bool[][],address[2],int192,uint8,bytes4,int8,int16,int128,uint72,uint128,uint224,bytes8,bytes19))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000ffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000200",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "cdToMem((int72,uint8,bool[][],address[2],int192,uint8,bytes4,int8,int16,int128,uint72,uint128,uint224,bytes8,bytes19))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000107",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000200",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "b4SlotRaw((int72,uint8,bool[][],address[2],int192,uint8,bytes4,int8,int16,int128,uint72,uint128,uint224,bytes8,bytes19))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000200",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000001",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "cdToMem((int72,uint8,bool[][],address[2],int192,uint8,bytes4,int8,int16,int128,uint72,uint128,uint224,bytes8,bytes19))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000200",
//!                         "0x0000000000000000000000010000000000000000000000000000000000000001",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "i16SlotRaw((int72,uint8,bool[][],address[2],int192,uint8,bytes4,int8,int16,int128,uint72,uint128,uint224,bytes8,bytes19))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000200",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                         "0xdeadbeef00000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
//!                         "0x0000000000000000000000000000000000000000000000000000000000008300",
//!                         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff91",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x000000000000000000000000000000000000000000000000000000000000006f",
//!                         "0x000000000000000000000000000000000000000000000000000000000000162e",
//!                         "0x0102030405060708000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd112233445566778899aabbccdd112200000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ],
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
pragma solidity ^0.8.0;

// Memory layout (32 bytes per field; reference types as pointer):
//   m+0x000: i72   (int72,   sign-extended)
//   m+0x020: u8    (uint8,   zero-padded)
//   m+0x040: flags (pointer to bool[][] heap area)
//   m+0x060: addrs (pointer to address[2] heap area)
//   m+0x080: i192  (int192,  sign-extended)
//   m+0x0a0: u8b   (uint8,   zero-padded)
//   m+0x0c0: b4    (bytes4,  left-aligned)
//   m+0x0e0: i8    (int8,    sign-extended)
//   m+0x100: i16   (int16,   sign-extended)
//   m+0x120: i128  (int128,  sign-extended)
//   m+0x140: u72   (uint72,  zero-padded)
//   m+0x160: u128  (uint128, zero-padded)
//   m+0x180: u224  (uint224, zero-padded)
//   m+0x1a0: b8    (bytes8,  left-aligned)
//   m+0x1c0: b19   (bytes19, left-aligned)
//
// Calldata ABI encoding (S is dynamic because of bool[][]):
//   word 0:  0x20  (outer offset to struct)
//   -- struct head (16 words) --
//   word 1:  i72  (int72, sign-extended)
//   word 2:  u8   (uint8)
//   word 3:  0x200 (offset to flags tail from struct start = 16 * 32)
//   word 4:  addrs[0] (address)
//   word 5:  addrs[1] (address)
//   word 6:  i192 (int192, sign-extended)
//   word 7:  u8b  (uint8)
//   word 8:  b4   (bytes4, left-aligned)
//   word 9:  i8   (int8, sign-extended)
//   word 10: i16  (int16, sign-extended)
//   word 11: i128 (int128, sign-extended)
//   word 12: u72  (uint72)
//   word 13: u128 (uint128)
//   word 14: u224 (uint224)
//   word 15: b8   (bytes8, left-aligned)
//   word 16: b19  (bytes19, left-aligned)
//   -- flags tail --
//   word 17: 0    (bool[][] outer length = 0)

struct S {
    int72        i72;
    uint8        u8;
    bool[][]     flags;
    address[2]   addrs;
    int192       i192;
    uint8        u8b;
    bytes4       b4;
    int8         i8;
    int16        i16;
    int128       i128;
    uint72       u72;
    uint128      u128;
    uint224      u224;
    bytes8       b8;
    bytes19      b19;
}

contract Test {
    // ── Signed-integer raw-slot tests (verify sign extension) ──────────────────

    // m+0x000: int72 i72=-1 → all 256 bits set.
    function i72SlotRaw(S calldata s) external pure returns (uint256 raw) {
        S memory m = s;
        assembly { raw := mload(m) }
    }

    // m+0x080: int192 i192=-1 → all 256 bits set.
    function i192SlotRaw(S calldata s) external pure returns (uint256 raw) {
        S memory m = s;
        assembly { raw := mload(add(m, 0x80)) }
    }

    // m+0x0e0: int8 i8=-128 → 0xFFFF...80.
    function i8SlotRaw(S calldata s) external pure returns (uint256 raw) {
        S memory m = s;
        assembly { raw := mload(add(m, 0xe0)) }
    }

    // m+0x100: int16 i16=-32000 (0x8300) → 0xFFFF...8300.
    function i16SlotRaw(S calldata s) external pure returns (uint256 raw) {
        S memory m = s;
        assembly { raw := mload(add(m, 0x100)) }
    }

    // ── Left-aligned bytesN raw-slot tests ───────────────────────────────────

    // m+0x0c0: bytes4 b4=0xDEADBEEF → 0xDEADBEEF followed by 28 zero bytes.
    function b4SlotRaw(S calldata s) external pure returns (uint256 raw) {
        S memory m = s;
        assembly { raw := mload(add(m, 0xc0)) }
    }

    // m+0x1a0: bytes8 b8=0x0102030405060708 → followed by 24 zero bytes.
    function b8SlotRaw(S calldata s) external pure returns (uint256 raw) {
        S memory m = s;
        assembly { raw := mload(add(m, 0x1a0)) }
    }

    // m+0x1c0: bytes19 b19=19-byte value → left-aligned, 13 trailing zero bytes.
    function b19SlotRaw(S calldata s) external pure returns (uint256 raw) {
        S memory m = s;
        assembly { raw := mload(add(m, 0x1c0)) }
    }

    // ── Typed round-trip: calldata → memory → ABI-encode ──────────────────────

    // Original signed/unsigned ints and bytes4 from the first half.
    function cdToMem(S calldata s)
        external pure
        returns (int72 i72, uint8 u8, int192 i192, uint8 u8b, bytes4 b4)
    {
        S memory m = s;
        return (m.i72, m.u8, m.i192, m.u8b, m.b4);
    }

    // Smaller signed ints, larger unsigned ints, and wider bytesN from the second half.
    function cdToMemExt(S calldata s)
        external pure
        returns (int8 i8, int16 i16, int128 i128,
                 uint72 u72, uint128 u128, uint224 u224,
                 bytes8 b8, bytes19 b19)
    {
        S memory m = s;
        return (m.i8, m.i16, m.i128, m.u72, m.u128, m.u224, m.b8, m.b19);
    }
}
