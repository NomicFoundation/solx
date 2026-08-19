//! { "cases": [ {
//!     "name": "zero_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "zero_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "-1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "zero_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
//!     ]
//! }, {
//!     "name": "zero_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "-632574534856236475345634624374238423192181237123712631236123123"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-632574534856236475345634624374238423192181237123712631236123123"
//!     ]
//! }, {
//!     "name": "zero_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!     ]
//! }, {
//!     "name": "zero_min",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0x8000000000000000000000000000000000000000000000000000000000000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x8000000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "one_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "-1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "one_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x29A44AC49A2C49A4AC81A482C81A482C81E6F9A1A21D59A1A21A1E661A1A1A21"
//!     ]
//! }, {
//!     "name": "one_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "-632574534856236475345634624374238423192181237123712631236123123"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-316287267428118237672817312187119211596090618561856315618061562"
//!     ]
//! }, {
//!     "name": "one_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x3FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!     ]
//! }, {
//!     "name": "one_min",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0x8000000000000000000000000000000000000000000000000000000000000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xC000000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "ordinary_bytes_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "40", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bytes_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "40", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bytes_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "40", "-1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "ordinary_bytes_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "40", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x00000000005348958934589349590349059034905903CDF343443AB34344343C"
//!     ]
//! }, {
//!     "name": "ordinary_bytes_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "40", "-632574534856236475345634624374238423192181237123712631236123123"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-575323187928221591706128151030714819346195542606177"
//!     ]
//! }, {
//!     "name": "ordinary_bytes_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "40", "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x00000000007FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!     ]
//! }, {
//!     "name": "ordinary_bytes_min",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "40", "0x8000000000000000000000000000000000000000000000000000000000000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xFFFFFFFFFF800000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "ordinary_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "171", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "171", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "171", "-1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "ordinary_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "171", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xA6912B1268B12692B2069"
//!     ]
//! }, {
//!     "name": "ordinary_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "171", "-632574534856236475345634624374238423192181237123712631236123123"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-211340361659"
//!     ]
//! }, {
//!     "name": "ordinary_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "171", "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x0000000000000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFF"
//!     ]
//! }, {
//!     "name": "ordinary_min",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "171", "0x8000000000000000000000000000000000000000000000000000000000000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-19342813113834066795298816"
//!     ]
//! }, {
//!     "name": "254_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "254", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "254_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "254", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "254_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "254", "-1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "254_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "254", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "254_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "254", "-632574534856236475345634624374238423192181237123712631236123123"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "254_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "254", "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "254_min",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "254", "0x8000000000000000000000000000000000000000000000000000000000000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-2"
//!     ]
//! }, {
//!     "name": "255_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "255", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "255_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "255", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "255_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "255", "-1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "255_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "255", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "255_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "255", "-632574534856236475345634624374238423192181237123712631236123123"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "255_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "255", "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "255_min",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "255", "0x8000000000000000000000000000000000000000000000000000000000000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "256_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "256", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "256_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "256", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "256_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "256", "-1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "256_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "256", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "256_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "256", "-632574534856236475345634624374238423192181237123712631236123123"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "256_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "256", "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "256_min",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "256", "0x8000000000000000000000000000000000000000000000000000000000000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "big_ordinary_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "35242523534534534233424343343443", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "35242523534534534233424343343443", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "35242523534534534233424343343443", "-1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "big_ordinary_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "35242523534534534233424343343443", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "35242523534534534233424343343443", "-632574534856236475345634624374238423192181237123712631236123123"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "big_ordinary_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "35242523534534534233424343343443", "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_min",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "35242523534534534233424343343443", "0x8000000000000000000000000000000000000000000000000000000000000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "max_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "-1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "max_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "-632574534856236475345634624374238423192181237123712631236123123"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "max_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_min",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0x8000000000000000000000000000000000000000000000000000000000000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.21;

contract Test {
    function main(uint256 a, uint256 b) external pure returns(uint256 result) {
        assembly {
            result := sar(a, b)
        }
    }
}
