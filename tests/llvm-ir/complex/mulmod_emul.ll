;! { "cases": [ {
;!     "name": "zero_zero_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "0", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "zero_one_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "1", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_zero_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_one_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "1", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "zero_ordinary_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "433478394034343", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_zero_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "121563127839120", "0", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_ordinary_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "433478394034343", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_one_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "121563127839120", "1", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "451167890987656789", "3456789009876545678", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_to_max_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x00000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0x0000000000000000000000000000000100000000000000000000000000000001", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_min_overflow_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x0000002000000000000000000000000000000000000000000000000000000000", "0x0000000000000000000000000000000000000000000000000000000008000000", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_overflow_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "2345678909876543212345678909876543212345678909876543", "23456789098765432123456789876543212345678987654321", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "zero_max_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_zero_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_max_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_one_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "1", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_max_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "7437834752357434334343423343443375834785783474", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_ordinary_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "23784273472384723848213821342323233223", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_max_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "zero_zero_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "0", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "zero_one_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "1", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_zero_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_one_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "1", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "zero_ordinary_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "433478394034343", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_zero_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "121563127839120", "0", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_ordinary_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "433478394034343", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_one_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "121563127839120", "1", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "two_one_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "2", "1", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "451167890987656789", "3456789009876545678", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_to_max_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x00000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0x0000000000000000000000000000000100000000000000000000000000000001", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_min_overflow_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x0000002000000000000000000000000000000000000000000000000000000000", "0x0000000000000000000000000000000000000000000000000000000008000000", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_overflow_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                "2345678909876543212345678909876543212345678909876543", "23456789098765432123456789876543212345678987654321", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "zero_max_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_zero_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_max_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_one_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "1", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_max_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "7437834752357434334343423343443375834785783474", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_ordinary_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "23784273472384723848213821342323233223", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_max_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "zero_zero_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "0", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "zero_one_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "1", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_zero_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_one_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "1", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "zero_ordinary_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "385347583459430234923842734623542325264782394023482374625423", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_zero_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "43873543534623235553423423423949234923947324723387478233", "0", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_ordinary_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "8939384548956875984554349903434903490994494993020959", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "8939384548956875984554349903434903490994494993020959"
;!     ]
;! }, {
;!     "name": "ordinary_one_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "54367829174652819374654281934765281937468197465281947562815652819", "1", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1122390563298694939482039005289855975809195069546337163"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "4511678909737783487438348343487656789", "3456789043333409876545678", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "5676430915488228986215146216393544180428262509017295078"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_ordinary_to_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "5893033507780600812692894395783280308047054170599805387", "18738036435609058585768", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_product_minus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "143295647605607314594356094730292530286858460075375207", "47", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_product",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "4", "1683723859365885946483684113080937230870586905885658682", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_product_plus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "2244965145821181261978245484107916307827449207847544909", "3", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "6734895437463543785934736452323748923482347623542634727"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_ordinary_bigger_product",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "4567892342342234534234", "3423423436457597665235", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "15637829699989597708005861762222266079154990"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_to_max_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x00000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
;!                 "0x0000000000000000000000000000000100000000000000000000000000000001",
;!                 "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "5205817307728658054337357463547686623259643155181961175"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_min_overflow_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x0000002000000000000000000000000000000000000000000000000000000000",
;!                 "0x0000000000000000000000000000000000000000000000000000000008000000",
;!                 "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "5205817307728658054337357463547686623259643155181961176"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_overflow_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "2345678909876543212345678909876543212345678909876543",
;!                 "23456789098765432123456789876543212345678987654321",
;!                 "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "4264809475174460139372937695722005330348657185160810887"
;!     ]
;! }, {
;!     "name": "zero_max_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_zero_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_max_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "5205817307728658054337357463547686623259643155181961175"
;!     ]
;! }, {
;!     "name": "max_one_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "1", "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "5205817307728658054337357463547686623259643155181961175"
;!     ]
;! }, {
;!     "name": "ordinary_max_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "7437834752357434334343423342323233245234534443443375834785783474",
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
;!                 "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "5617213867212480757578809585112783067547734929923987190"
;!     ]
;! }, {
;!     "name": "max_ordinary_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
;!                 "23784273472384723848213821342323233223",
;!                 "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "3578153470724301227568936858877098024097400680412702913"
;!     ]
;! }, {
;!     "name": "max_max_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
;!                 "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1846270932306819610647429224033438799896242185499682409"
;!     ]
;! }, {
;!     "name": "zero_zero_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "0", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "zero_one_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "1", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_zero_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_one_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "1", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "zero_ordinary_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "385347583459430234923842734623542325264782394023482374625423", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_zero_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "43873543534623235553423423423949234923947324723387478233", "0", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_ordinary_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "8939384548956875984554349903434903490994494993020959", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "8939384548956875984554349903434903490994494993020959"
;!     ]
;! }, {
;!     "name": "ordinary_one_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "54367829174652819374654281934765281937468197465281947562815652819", "1", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "54367829174652819374654281934765281937468197465281947562815652819"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "4511678909737783487438348343487656789", "3456789043333409876545678", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "15595922222219994270792624200954893890665171031928705745307942"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_to_max_minus_one_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "14",
;!                 "8270863516951156815969356072049136275233570333260040288532684571993794974281",
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_to_max_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x00000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
;!                 "0x0000000000000000000000000000000100000000000000000000000000000001",
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_min_overflow_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x0000002000000000000000000000000000000000000000000000000000000000",
;!                 "0x0000000000000000000000000000000000000000000000000000000008000000",
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_overflow_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "2345678909876543212345678909876543212345678909876543",
;!                 "23456789098765432123456789876543212345678987654321",
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "109870817467274783132168917047078478060902558898874810394629740062300719137698"
;!     ]
;! }, {
;!     "name": "zero_max_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_zero_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_max_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_one_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "1", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_max_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "7437834752357434334343423342323233245234534443443375834785783474",
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_ordinary_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
;!                 "23784273472384723848213821342323233223",
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_max_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! } ] }

target datalayout = "E-p:256:256-i256:256:256-S256-a:256:256"
target triple = "evm-unknown-unknown"

define void @test() noreturn {
bb:
  %off1 = inttoptr i256 0 to ptr addrspace(2)
  %off2 = inttoptr i256 32 to ptr addrspace(2)
  %off3 = inttoptr i256 64 to ptr addrspace(2)
  %arg1 = call i256 @llvm.evm.calldataload(ptr addrspace(2) %off1)
  %arg2 = call i256 @llvm.evm.calldataload(ptr addrspace(2) %off2)
  %arg3 = call i256 @llvm.evm.calldataload(ptr addrspace(2) %off3)
  %res = call i256 @mulmod(i256 noundef %arg1, i256 noundef %arg2, i256 noundef %arg3)
  store i256 %res, ptr addrspace(1) null, align 4
  call void @llvm.evm.return(ptr addrspace(1) null, i256 32)
  unreachable
}

define private i256 @clz(i256 %v) noinline {
entry:
  %vs128 = lshr i256 %v, 128
  %vs128nz = icmp ne i256 %vs128, 0
  %n128 = select i1 %vs128nz, i256 128, i256 256
  %va128 = select i1 %vs128nz, i256 %vs128, i256 %v
  %vs64 = lshr i256 %va128, 64
  %vs64nz = icmp ne i256 %vs64, 0
  %clza64 = sub i256 %n128, 64
  %n64 = select i1 %vs64nz, i256 %clza64, i256 %n128
  %va64 = select i1 %vs64nz, i256 %vs64, i256 %va128
  %vs32 = lshr i256 %va64, 32
  %vs32nz = icmp ne i256 %vs32, 0
  %clza32 = sub i256 %n64, 32
  %n32 = select i1 %vs32nz, i256 %clza32, i256 %n64
  %va32 = select i1 %vs32nz, i256 %vs32, i256 %va64
  %vs16 = lshr i256 %va32, 16
  %vs16nz = icmp ne i256 %vs16, 0
  %clza16 = sub i256 %n32, 16
  %n16 = select i1 %vs16nz, i256 %clza16, i256 %n32
  %va16 = select i1 %vs16nz, i256 %vs16, i256 %va32
  %vs8 = lshr i256 %va16, 8
  %vs8nz = icmp ne i256 %vs8, 0
  %clza8 = sub i256 %n16, 8
  %n8 = select i1 %vs8nz, i256 %clza8, i256 %n16
  %va8 = select i1 %vs8nz, i256 %vs8, i256 %va16
  %vs4 = lshr i256 %va8, 4
  %vs4nz = icmp ne i256 %vs4, 0
  %clza4 = sub i256 %n8, 4
  %n4 = select i1 %vs4nz, i256 %clza4, i256 %n8
  %va4 = select i1 %vs4nz, i256 %vs4, i256 %va8
  %vs2 = lshr i256 %va4, 2
  %vs2nz = icmp ne i256 %vs2, 0
  %clza2 = sub i256 %n4, 2
  %n2 = select i1 %vs2nz, i256 %clza2, i256 %n4
  %va2 = select i1 %vs2nz, i256 %vs2, i256 %va4
  %vs1 = lshr i256 %va2, 1
  %vs1nz = icmp ne i256 %vs1, 0
  %clza1 = sub i256 %n2, 2
  %clzax = sub i256 %n2, %va2
  %result = select i1 %vs1nz, i256 %clza1, i256 %clzax
  ret i256 %result
}

define private i256 @ulongrem(i256 %0, i256 %1, i256 %2) noinline {
  %.not = icmp ult i256 %1, %2
  br i1 %.not, label %4, label %51

4:
  %5 = tail call i256 @clz(i256 %2)
  %.not61 = icmp eq i256 %5, 0
  br i1 %.not61, label %13, label %6

6:
  %7 = shl i256 %2, %5
  %8 = shl i256 %1, %5
  %9 = sub nuw nsw i256 256, %5
  %10 = lshr i256 %0, %9
  %11 = or i256 %10, %8
  %12 = shl i256 %0, %5
  br label %13

13:
  %.054 = phi i256 [ %7, %6 ], [ %2, %4 ]
  %.053 = phi i256 [ %11, %6 ], [ %1, %4 ]
  %.052 = phi i256 [ %12, %6 ], [ %0, %4 ]
  %14 = lshr i256 %.054, 128
  %15 = udiv i256 %.053, %14
  %16 = urem i256 %.053, %14
  %17 = and i256 %.054, 340282366920938463463374607431768211455
  %18 = lshr i256 %.052, 128
  br label %19

19:
  %.056 = phi i256 [ %15, %13 ], [ %25, %.critedge ]
  %.055 = phi i256 [ %16, %13 ], [ %26, %.critedge ]
  %.not62 = icmp ult i256 %.056, 340282366920938463463374607431768211455
  br i1 %.not62, label %20, label %.critedge

20:
  %21 = mul nuw i256 %.056, %17
  %22 = shl nuw i256 %.055, 128
  %23 = or i256 %22, %18
  %24 = icmp ugt i256 %21, %23
  br i1 %24, label %.critedge, label %27

.critedge:
  %25 = add i256 %.056, -1
  %26 = add i256 %.055, %14
  %.not65 = icmp ult i256 %26, 340282366920938463463374607431768211455
  br i1 %.not65, label %19, label %27

27:
  %.157 = phi i256 [ %25, %.critedge ], [ %.056, %20 ]
  %28 = shl i256 %.053, 128
  %29 = or i256 %18, %28
  %30 = and i256 %.157, 340282366920938463463374607431768211455
  %31 = mul i256 %30, %.054
  %32 = sub i256 %29, %31
  %33 = udiv i256 %32, %14
  %34 = urem i256 %32, %14
  %35 = and i256 %.052, 340282366920938463463374607431768211455
  br label %36

36:
  %.2 = phi i256 [ %33, %27 ], [ %42, %.critedge1 ]
  %.1 = phi i256 [ %34, %27 ], [ %43, %.critedge1 ]
  %.not63 = icmp ult i256 %.2, 340282366920938463463374607431768211455
  br i1 %.not63, label %37, label %.critedge1

37:
  %38 = mul nuw i256 %.2, %17
  %39 = shl i256 %.1, 128
  %40 = or i256 %39, %35
  %41 = icmp ugt i256 %38, %40
  br i1 %41, label %.critedge1, label %44

.critedge1:
  %42 = add i256 %.2, -1
  %43 = add i256 %.1, %14
  %.not64 = icmp ult i256 %43, 340282366920938463463374607431768211455
  br i1 %.not64, label %36, label %44

44:
  %.3 = phi i256 [ %42, %.critedge1 ], [ %.2, %37 ]
  %45 = shl i256 %32, 128
  %46 = or i256 %45, %35
  %47 = and i256 %.3, 340282366920938463463374607431768211455
  %48 = mul i256 %47, %.054
  %49 = sub i256 %46, %48
  %50 = lshr i256 %49, %5
  br label %51

51:
  %.0 = phi i256 [ %50, %44 ], [ -1, %3 ]
  ret i256 %.0
}

define private i256 @mulmod(i256 %arg1, i256 %arg2, i256 %modulo) noinline {
entry:
  %cccond = icmp eq i256 %modulo, 0
  br i1 %cccond, label %ccret, label %entrycont
ccret:
  ret i256 0
entrycont:
  %arg1m = urem i256 %arg1, %modulo
  %arg2m = urem i256 %arg2, %modulo
  %less_then_2_128 = icmp ult i256 %modulo, 340282366920938463463374607431768211456
  br i1 %less_then_2_128, label %fast, label %slow
fast:
  %prod = mul i256 %arg1m, %arg2m
  %prodm = urem i256 %prod, %modulo
  ret i256 %prodm
slow:
  %arg1e = zext i256 %arg1m to i512
  %arg2e = zext i256 %arg2m to i512
  %prode = mul i512 %arg1e, %arg2e
  %prodl = trunc i512 %prode to i256
  %prodeh = lshr i512 %prode, 256
  %prodh = trunc i512 %prodeh to i256
  %res = call i256 @ulongrem(i256 %prodl, i256 %prodh, i256 %modulo)
  ret i256 %res
}

declare {i256, i1} @llvm.uadd.with.overflow.i256(i256, i256)
declare void @llvm.evm.return(ptr addrspace(1), i256)
declare i256 @llvm.evm.calldataload(ptr addrspace(2))
