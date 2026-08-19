//! { "cases": [ {
//!     "name": "zero_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0",
//!                 "0",
//!                 "0",
//!                 "0",
//!                 "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0",
//!                     "0",
//!                     "0",
//!                     "0"
//!                 ],
//!                 "values": [
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "zero_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "1",
//!                 "1",
//!                 "0",
//!                 "1",
//!                 "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "1",
//!                     "0",
//!                     "1",
//!                     "0"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "zero_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "19",
//!                 "1",
//!                 "1",
//!                 "1",
//!                 "1"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "1",
//!                     "1",
//!                     "1",
//!                     "1"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "zero_word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "32",
//!                 "0",
//!                 "7873277347887348384828938912903901239012089389127312737120012",
//!                 "7873277347887348384828938912903901239012089389127312737120012",
//!                 "7873277347887348384828938912903901239012089389127312737120012"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0",
//!                     "7873277347887348384828938912903901239012089389127312737120012",
//!                     "7873277347887348384828938912903901239012089389127312737120012",
//!                     "7873277347887348384828938912903901239012089389127312737120012"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "zero_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "8340",
//!                 "7873277347887348384828938912903901239012089389127312737120012",
//!                 "7873277347887348384828938912903901239012089389127312737120012",
//!                 "1",
//!                 "1"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "7873277347887348384828938912903901239012089389127312737120012",
//!                     "7873277347887348384828938912903901239012089389127312737120012",
//!                     "1",
//!                     "1"
//!                 ],
//!                 "values": [
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "zero_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0x100",
//!                 "7873277347887348384828938912903901239012089389127312737120012",
//!                 "7873277347887348384828938912903901239012089389127312737120012",
//!                 "7873277347887348384828938912903901239012089389127312737120012",
//!                 "7873277347887348384828938912903901239012089389127312737120012"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "7873277347887348384828938912903901239012089389127312737120012",
//!                     "7873277347887348384828938912903901239012089389127312737120012",
//!                     "7873277347887348384828938912903901239012089389127312737120012",
//!                     "7873277347887348384828938912903901239012089389127312737120012"
//!                 ],
//!                 "values": [
//!                     "0", "0", "0", "0", "0", "0", "0", "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "zero_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0xFFFFFFAA",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "zero_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0xFFFFFFE0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "zero_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0xFFFFFFE1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "zero_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0xFFFFFFEF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "zero_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0xFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "zero_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0x100000000",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "zero_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "3334353453452342342354355544445321191012012",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "zero_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "one_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0",
//!                 "0",
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0"
//!                 ],
//!                 "values": [
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "one_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "1",
//!                 "1",
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "1",
//!                 "1"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "1",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "1",
//!                     "1"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "one_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "19",
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "7873277347887348384828938912903901239012089389127312737120012",
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "7873277347887348384828938912903901239012089389127312737120012",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "one_word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "32",
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "one_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "8340",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "one_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0x100",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0", "0", "0", "0", "0", "0", "0", "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "one_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0xFFFFFFAA",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "one_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0xFFFFFFE0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "one_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0xFFFFFFE1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "one_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0xFFFFFFEF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "one_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0xFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "one_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0x100000000",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "one_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "3334353453452342342354355544445321191012012",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "one_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "small_ordinary_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "8", "0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "small_ordinary_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "8", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "small_ordinary_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "8", "19",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "small_ordinary_word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "8", "32",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "small_ordinary_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "8", "8340",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "small_ordinary_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "8", "0x100",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0", "0", "0", "0", "0", "0", "0", "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "small_ordinary_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "8", "0xFFFFFFAA",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "small_ordinary_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "8", "0xFFFFFFE0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "small_ordinary_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "8", "0xFFFFFFE1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "small_ordinary_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "8", "0xFFFFFFEF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "small_ordinary_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "8", "0xFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "small_ordinary_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "8", "0x100000000",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "small_ordinary_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "8", "3334353453452342342354355544445321191012012",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "small_ordinary_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "8", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "word_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "word_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "word_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "19",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "word_word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "32",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "word_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "8340",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "word_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "0x100",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0", "0", "0", "0", "0", "0", "0", "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "word_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "0xFFFFFFAA",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "word_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "0xFFFFFFE0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "word_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "0xFFFFFFE1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "word_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "0xFFFFFFEF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "word_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "0xFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "word_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "0x100000000",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "word_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "3334353453452342342354355544445321191012012",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "word_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "421587", "0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "421587", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "421587", "19",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "421587", "32",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "421587", "8340",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "421587", "0x100",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0", "0", "0", "0", "0", "0", "0", "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "421587", "0xFFFFFFAA",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "421587", "0xFFFFFFE0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "421587", "0xFFFFFFE1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "421587", "0xFFFFFFEF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "421587", "0xFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "421587", "0x100000000",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "421587", "3334353453452342342354355544445321191012012",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "421587", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_words_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_words_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_words_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "19",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_words_word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "32",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_words_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "8340",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
//!                     "0", "0", "0", "0", "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_words_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "0x100",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0", "0", "0", "0", "0", "0", "0", "0"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_words_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "0xFFFFFFAA",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_words_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "0xFFFFFFE0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_words_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "0xFFFFFFE1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_words_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "0xFFFFFFEF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_words_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "0xFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_words_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "0x100000000",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_words_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "3334353453452342342354355544445321191012012",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_words_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "big_ordinary_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": false,
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "address": "Test.address",
//!                 "topics": [
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a"
//!                 ],
//!                 "values": []
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "big_ordinary_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "big_ordinary_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "19",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "big_ordinary_word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "32",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "big_ordinary_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "8340",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "big_ordinary_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0x100",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "big_ordinary_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0xFFFFFFAA",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "big_ordinary_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0xFFFFFFE0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "big_ordinary_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0xFFFFFFE1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "big_ordinary_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0xFFFFFFEF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "big_ordinary_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0xFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "big_ordinary_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0x100000000",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "big_ordinary_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "3334353453452342342354355544445321191012012",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "big_ordinary_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": false,
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "address": "Test.address",
//!                 "topics": [
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a"
//!                 ],
//!                 "values": []
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "19",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "32",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "8340",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0x100",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0xFFFFFFAA",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0xFFFFFFE0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0xFFFFFFE1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0xFFFFFFEF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0xFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0x100000000",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "3334353453452342342354355544445321191012012",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": false,
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "address": "Test.address",
//!                 "topics": [
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a"
//!                 ],
//!                 "values": []
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "19",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "32",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "8340",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0x100",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0xFFFFFFAA",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0xFFFFFFE0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0xFFFFFFE1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0xFFFFFFEF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0xFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0x100000000",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "3334353453452342342354355544445321191012012",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": false,
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "address": "Test.address",
//!                 "topics": [
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a"
//!                 ],
//!                 "values": []
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "19",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "32",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "8340",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0x100",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0xFFFFFFAA",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0xFFFFFFE0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0xFFFFFFE1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0xFFFFFFEF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0xFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0x100000000",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "3334353453452342342354355544445321191012012",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": false,
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "address": "Test.address",
//!                 "topics": [
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a"
//!                 ],
//!                 "values": []
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "19",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "32",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "8340",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0x100",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0xFFFFFFAA",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0xFFFFFFE0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0xFFFFFFE1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0xFFFFFFEF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0xFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0x100000000",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "3334353453452342342354355544445321191012012",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": false,
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "address": "Test.address",
//!                 "topics": [
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a"
//!                 ],
//!                 "values": []
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "2_in_32_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "19",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "32",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "8340",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "0x100",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "0xFFFFFFAA",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "0xFFFFFFE0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "0xFFFFFFE1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "0xFFFFFFEF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "0xFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "0x100000000",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "3334353453452342342354355544445321191012012",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": false,
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "address": "Test.address",
//!                 "topics": [
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a"
//!                 ],
//!                 "values": []
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "19",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "32",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "8340",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0x100",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0xFFFFFFAA",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0xFFFFFFE0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0xFFFFFFE1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0xFFFFFFEF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0xFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0x100000000",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "3334353453452342342354355544445321191012012",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": false,
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "address": "Test.address",
//!                 "topics": [
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a",
//!                     "0xa348384f3434bc232d4433aaf3232ccbfffee3454ff33545ccd4343f121d434a"
//!                 ],
//!                 "values": []
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "max_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "19",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "32",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "8340",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0x100",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xFFFFFFAA",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xFFFFFFE0",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xFFFFFFE1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xFFFFFFEF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0x100000000",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "3334353453452342342354355544445321191012012",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "one_value_one_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "0", "1", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0x0100000000000000000000000000000000000000000000000000000000000000"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "one_value_small_ordinary_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "123122", "19", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0x0101010101010101010101010101010101010100000000000000000000000000"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "one_value_word_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "12", "32", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0x0101010101010101010101010101010101010101010101010101010101010101"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "one_value_ordinary_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "1024", "273", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0x0101010101010101010101010101010101010101010101010101010101010101",
//!                     "0x0101010101010101010101010101010101010101010101010101010101010101",
//!                     "0x0101010101010101010101010101010101010101010101010101010101010101",
//!                     "0x0101010101010101010101010101010101010101010101010101010101010101",
//!                     "0x0101010101010101010101010101010101010101010101010101010101010101",
//!                     "0x0101010101010101010101010101010101010101010101010101010101010101",
//!                     "0x0101010101010101010101010101010101010101010101010101010101010101",
//!                     "0x0101010101010101010101010101010101010101010101010101010101010101",
//!                     "0x0101010101010101010101010101010101000000000000000000000000000000"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "one_value_ordinary_words_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "1", "64", "1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0x0101010101010101010101010101010101010101010101010101010101010101",
//!                     "0x0101010101010101010101010101010101010101010101010101010101010101"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_value_one_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "0", "1", "0xA1",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0xA100000000000000000000000000000000000000000000000000000000000000"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_value_small_ordinary_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "123122", "19", "0x44",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0x4444444444444444444444444444444444444400000000000000000000000000"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_value_word_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "12", "32", "0xE6",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0xE6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6E6"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_value_ordinary_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "1024", "273", "0xD5",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0xD5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5",
//!                     "0xD5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5",
//!                     "0xD5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5",
//!                     "0xD5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5",
//!                     "0xD5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5",
//!                     "0xD5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5",
//!                     "0xD5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5",
//!                     "0xD5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5",
//!                     "0xD5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5D5000000000000000000000000000000"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "ordinary_value_ordinary_words_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "1", "64", "0x82",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0x8282828282828282828282828282828282828282828282828282828282828282",
//!                     "0x8282828282828282828282828282828282828282828282828282828282828282"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "max_value_one_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "0", "1", "0xFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0xFF00000000000000000000000000000000000000000000000000000000000000"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "max_value_small_ordinary_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "123122", "19", "0xFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF00000000000000000000000000"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "max_value_word_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "12", "32", "0xFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "max_value_ordinary_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "1024", "273", "0xFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF000000000000000000000000000000"
//!                 ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "max_value_ordinary_words_len",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "1", "64", "0xFF",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                 "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!         ],
//!         "events": [
//!             {
//!                 "topics": [
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A",
//!                     "0xA348384F3434BC232D4433AAF3232CCBFFFEE3454FF33545CCD4343F121D434A"
//!                 ],
//!                 "values": [
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                     "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!                 ]
//!             }
//!         ]
//!     }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(uint256 offset, uint256 len, uint256 t1, uint256 t2, uint256 t3, uint256 t4) external {
        assembly {
            // nullify memory ptr slot
            mstore(0x40, 0)
            log4(offset, len, t1, t2, t3, t4)
        }
    }

    function with_value(uint256 offset, uint256 len, uint8 val, uint256 t1, uint256 t2, uint256 t3, uint256 t4) external {
        assembly {
            for { let i := 0 } lt(i, len) { i := add(i, 1) }
            {
                mstore8(add(offset, i), val)
            }
            log4(offset, len, t1, t2, t3, t4)
            // TODO: can be removed after the VM fix
            return(0, 0)
        }
    }
}
