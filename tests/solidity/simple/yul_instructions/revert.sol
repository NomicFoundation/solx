//! { "cases": [ {
//!     "name": "zero_zero",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "zero_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "zero_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0", "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "zero_word",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0", "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "zero_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0", "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "zero_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0", "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "zero_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0", "0xFFFFFFAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0", "0xFFFFFFE0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0", "0xFFFFFFE1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0", "0xFFFFFFEF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0", "0xFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0", "0x100000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0", "3334353453452342342354355544445321191012012"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_max",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_zero",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "1", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "one_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "1", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "one_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "1", "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "one_word",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "1", "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "one_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "1", "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "one_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "1", "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "one_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "1", "0xFFFFFFAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "1", "0xFFFFFFE0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "1", "0xFFFFFFE1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "1", "0xFFFFFFEF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "1", "0xFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "1", "0x100000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "1", "3334353453452342342354355544445321191012012"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_max",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "1", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "small_ordinary_zero",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "8", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "small_ordinary_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "8", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "small_ordinary_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "8", "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "small_ordinary_word",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "8", "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "small_ordinary_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "8", "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "small_ordinary_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "8", "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "small_ordinary_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "8", "0xFFFFFFAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "small_ordinary_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "8", "0xFFFFFFE0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "small_ordinary_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "8", "0xFFFFFFE1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "small_ordinary_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "8", "0xFFFFFFEF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "small_ordinary_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "8", "0xFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "small_ordinary_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "8", "0x100000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "small_ordinary_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "8", "3334353453452342342354355544445321191012012"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "small_ordinary_max",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "8", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "word_zero",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "32", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "word_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "32", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "word_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "32", "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "word_word",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "32", "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "word_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "32", "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "word_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "32", "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "word_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "32", "0xFFFFFFAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "word_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "32", "0xFFFFFFE0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "word_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "32", "0xFFFFFFE1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "word_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "32", "0xFFFFFFEF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "word_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "32", "0xFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "word_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "32", "0x100000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "word_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "32", "3334353453452342342354355544445321191012012"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "word_max",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "32", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_zero",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "421587", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "421587", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "421587", "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_word",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "421587", "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "421587", "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "421587", "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "421587", "0xFFFFFFAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "421587", "0xFFFFFFE0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "421587", "0xFFFFFFE1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "421587", "0xFFFFFFEF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "421587", "0xFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "421587", "0x100000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "421587", "3334353453452342342354355544445321191012012"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_max",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "421587", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_words_zero",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xF100", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_words_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xF100", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_words_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xF100", "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_words_word",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xF100", "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_words_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xF100", "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_words_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xF100", "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_words_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xF100", "0xFFFFFFAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_words_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xF100", "0xFFFFFFE0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_words_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xF100", "0xFFFFFFE1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_words_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xF100", "0xFFFFFFEF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_words_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xF100", "0xFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_words_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xF100", "0x100000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_words_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xF100", "3334353453452342342354355544445321191012012"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_words_max",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xF100", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_zero",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "big_ordinary_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFAA", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFAA", "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_word",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFAA", "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFAA", "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0xFFFFFFAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0xFFFFFFE0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0xFFFFFFE1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0xFFFFFFEF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0xFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0x100000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFAA", "3334353453452342342354355544445321191012012"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary_max",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_32_zero",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_32_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE0", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_32_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE0", "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_32_word",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE0", "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_32_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE0", "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_32_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_32_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0xFFFFFFAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_32_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0xFFFFFFE0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_32_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0xFFFFFFE1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_32_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0xFFFFFFEF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_32_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0xFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_32_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0x100000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_32_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE0", "3334353453452342342354355544445321191012012"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_32_max",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_31_zero",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_31_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE1", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_31_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE1", "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_31_word",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE1", "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_31_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE1", "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_31_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_31_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0xFFFFFFAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_31_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0xFFFFFFE0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_31_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0xFFFFFFE1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_31_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0xFFFFFFEF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_31_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0xFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_31_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0x100000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_31_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE1", "3334353453452342342354355544445321191012012"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_31_max",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_ordinary_zero",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_ordinary_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFEF", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_ordinary_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFEF", "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_ordinary_word",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFEF", "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_ordinary_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFEF", "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_ordinary_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_ordinary_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0xFFFFFFAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_ordinary_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0xFFFFFFE0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_ordinary_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0xFFFFFFE1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_ordinary_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0xFFFFFFEF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_ordinary_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0xFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_ordinary_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0x100000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_ordinary_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFEF", "3334353453452342342354355544445321191012012"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_ordinary_max",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_one_zero",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_one_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFF", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_one_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFF", "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_one_word",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFF", "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_one_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFF", "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_one_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_one_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0xFFFFFFAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_one_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0xFFFFFFE0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_one_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0xFFFFFFE1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_one_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0xFFFFFFEF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_one_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0xFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_one_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0x100000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_one_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFF", "3334353453452342342354355544445321191012012"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_minus_one_max",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_zero",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0x100000000", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "2_in_32_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0x100000000", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0x100000000", "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_word",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0x100000000", "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0x100000000", "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0x100000000", "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0x100000000", "0xFFFFFFAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0x100000000", "0xFFFFFFE0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0x100000000", "0xFFFFFFE1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0x100000000", "0xFFFFFFEF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0x100000000", "0xFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0x100000000", "0x100000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0x100000000", "3334353453452342342354355544445321191012012"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "2_in_32_max",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0x100000000", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_32_zero",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_32_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_32_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_32_word",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_32_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_32_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_32_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0xFFFFFFAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_32_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0xFFFFFFE0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_32_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0xFFFFFFE1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_32_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0xFFFFFFEF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_32_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0xFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_32_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0x100000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_32_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "3334353453452342342354355544445321191012012"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_32_max",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_zero",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "max_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_word",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xFFFFFFAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xFFFFFFE0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xFFFFFFE1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xFFFFFFEF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0x100000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "3334353453452342342354355544445321191012012"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_max",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_value_one_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "0", "1", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "one_value_small_ordinary_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "123122", "19", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "one_value_word_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "12", "32", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "one_value_ordinary_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "1024", "2383", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "one_value_ordinary_words_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "1", "0xF100", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_value_one_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "0", "1", "0xA1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_value_small_ordinary_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "123122", "19", "0x44"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_value_word_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "12", "32", "0xE6"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_value_ordinary_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "1024", "2383", "0xD5"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "ordinary_value_ordinary_words_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "1", "0xF100", "0x82"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "max_value_one_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "0", "1", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "max_value_small_ordinary_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "123122", "19", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "max_value_word_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "12", "32", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "max_value_ordinary_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "1024", "2383", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! }, {
//!     "name": "max_value_ordinary_words_len",
//!     "inputs": [
//!         {
//!             "method": "test_with_value",
//!             "calldata": [
//!                 "1", "0xF100", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.21;

contract Test {
    function test(uint256 offset, uint256 len) external {
        assembly {
            mstore(0, 0xcc572cf9) // main selector
            mstore(32, offset)
            mstore(64, len)
            let success := call(gas(), address(), 0, 28, 68, 0, 0)

            mstore(0, success)
            returndatacopy(32, 0, returndatasize())
            if gt(success, 0) {
                return(0, add(returndatasize(), 32))
            }

            if iszero(eq(len, returndatasize())) {
                return(0, add(returndatasize(), 32))
            }

            for { let i := 0 } lt(i, len) { i := add(i, 1) }
            {
                if iszero(eq(shr(248, mload(add(i, 32))), 0)) {
                    return(0, add(returndatasize(), 32))
                }
            }

            // return some magic value in a success case
            mstore(0, 0xdeadbeef)
            return(0, 32)
        }
    }

    function test_with_value(uint256 offset, uint256 len, uint8 val) external {
        assembly {
            mstore(0, 0xc9b9b445) // with_value selector
            mstore(32, offset)
            mstore(64, len)
            mstore(96, val)
            let success := call(gas(), address(), 0, 28, 100, 32, len)

            mstore(0, success)
            if gt(success, 0) {
                return(0, add(returndatasize(), 32))
            }

            if iszero(eq(len, returndatasize())) {
                return(0, add(returndatasize(), 32))
            }

            for { let i := 0 } lt(i, len) { i := add(i, 1) }
            {
                if iszero(eq(shr(248, mload(add(i, 32))), val)) {
                    return(0, add(returndatasize(), 32))
                }
            }

            // return some magic value in a success case
            mstore(0, 0xdeadbeef)
            return(0, 32)
        }
    }

    function main(uint256 offset, uint256 len) external pure {
        assembly {
            // nullify memory ptr slot
            mstore(0x40, 0)
            revert(offset, len)
        }
    }

    function with_value(uint256 offset, uint256 len, uint8 val) external view {
        assembly {
            for { let i := 0 } lt(i, len) { i := add(i, 1) }
            {
                mstore8(add(offset, i), val)
            }
            revert(offset, len)
        }
    }
}
