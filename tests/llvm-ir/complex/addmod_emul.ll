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
;!                 "0xfafffffffffffffffffffffffffffffffffffffffffffffffffffffff4ffffff", "0x050000000000000000000000000000000000000000000000000000000b000000", "0"
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
;!                 "0xab000000000000000000000000000000000000000000000000000000000000a1", "0x54ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff5f", "0"
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
;!                 "0x2fff1ffffffffff5ffffff0fffffffff2ffffffafffafffcffff1ff234ffffff", "0xef231ffffffffff4f12fff34ffffffff2fffffbbfffafffdffff22f538ffffff", "0"
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
;!     "name": "two_zero_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "2", "0", "1"
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
;!                 "0xfafffffffffffffffffffffffffffffffffffffffffffffffffffffff4ffffff", "0x050000000000000000000000000000000000000000000000000000000b000000", "1"
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
;!                 "0xab000000000000000000000000000000000000000000000000000000000000a1", "0x54ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff5f", "1"
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
;!                 "0x2fff1ffffffffff5ffffff0fffffffff2ffffffafffafffcffff1ff234ffffff", "0xef231ffffffffff4f12fff34ffffffff2fffffbbfffafffdffff22f538ffffff", "1"
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
;!         "1"
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
;!         "1"
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
;!         "2"
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
;!         "3806109516113667800853767386706858816392394866986028175"
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
;!         "3464170909841972837815004710006741383053238982131669865"
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
;!         "8939384548956875984554349903434903490994494993020960"
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
;!         "1122390563298694939482039005289855975809195069546337164"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "8934583485834958423984923948923489238948923232849429834",
;!                 "5743892143323342342342342342343234234234234233423432423",
;!                 "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1208684754231213194457793386619225626218462219187592801"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_ordinary_to_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "67345763475734785873485834858348958348582848238423848",
;!                 "47076922298769071715669669331407893506027850516560019248",
;!                 "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_sum_minus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "2627823478238423324723748234889489238948234823848237723",
;!                 "4107071959225120461210988217434259684534112799694397006",
;!                 "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_sum",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "78534387453485348583489583489583495834858934985982398",
;!                 "6656361050010058437351246868834165427647488688556652330",
;!                 "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_sum_plus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "2745764578678457867485678478567845576875845678454554545",
;!                 "3989130858785085918449057973755903346606501945088080182",
;!                 "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "6734895437463543785934736452323748923482347623542634727"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_ordinary_bigger_sum",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "4567890987654678923042342342342342234534234",
;!                 "34234234322332342356457567897665235",
;!                 "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "4567891021888913245374684698799910132199469"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_to_max_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xfafffffffffffffffffffffffffffffffffffffffffffffffffffffff4ffffff",
;!                 "0x050000000000000000000000000000000000000000000000000000000b000000",
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
;!                 "0xab000000000000000000000000000000000000000000000000000000000000a1",
;!                 "0x54ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff5f",
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
;!                 "0x2fff1ffffffffff5ffffff0fffffffff2ffffffafffafffcffff1ff234ffffff",
;!                 "0xef231ffffffffff4f12fff34ffffffff2fffffbbfffafffdffff22f538ffffff",
;!                 "6734895437463543785934736452323748923482347623542634728"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "143268280533798984280626563216701884452898533151473998"
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
;!         "5205817307728658054337357463547686623259643155181961175"
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
;!         "5205817307728658054337357463547686623259643155181961175"
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
;!         "5205817307728658054337357463547686623259643155181961176"
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
;!         "5205817307728658054337357463547686623259643155181961176"
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
;!         "4785420515141501473933080503501442490599789180695732857"
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
;!         "5205817307728658078121630935932410471473464497505194398"
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
;!         "3676739177993772322739978474771624323036938686821287622"
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
;!         "1"
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
;!         "1"
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
;!         "2"
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
;!         "385347583459430234923842734623542325264782394023482374625423"
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
;!         "43873543534623235553423423423949234923947324723387478233"
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
;!         "8939384548956875984554349903434903490994494993020960"
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
;!         "54367829174652819374654281934765281937468197465281947562815652820"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "8934583485834958423984923948923489238948923232849429834",
;!                 "5743892143323342342342342342343234234234234233423432423",
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "14678475629158300766327266291266723473183157466272862257"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_to_max_minus_one_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "67345763475737323342423423423423432232333434485834858348958348582848238423848",
;!                 "48446325761578872081147561585264475620936550179805705690499235425064891216086",
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
;!                 "2627823478238423324723748234889489238948234823848237723",
;!                 "115792089237316195423568357185209669429945260917405674550218635773089281402212",
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
;!                 "0xab000000000000000000000000000000000000000000000000000000000000a1",
;!                 "0x54ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff5f",
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
;!                 "0x2fff1ffffffffff5ffffff0fffffffff2ffffffafffafffcffff1ff234ffffff",
;!                 "0xef231ffffffffff4f12fff34ffffffff2fffffbbfffafffdffff22f538ffffff",
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "14082212818049917570678938371545509262648545780014710136968537875118318157823"
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
;!         "1"
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
;!         "1"
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
;!         "7437834752357434334343423342323233245234534443443375834785783474"
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
;!         "23784273472384723848213821342323233223"
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
  %res = call i256 @addmod(i256 noundef %arg1, i256 noundef %arg2, i256 noundef %arg3)
  store i256 %res, ptr addrspace(1) null, align 4
  call void @llvm.evm.return(ptr addrspace(1) null, i256 32)
  unreachable
}

define i256 @addmod(i256 %arg1, i256 %arg2, i256 %modulo) #0 {
entry:
  %is_zero = icmp eq i256 %modulo, 0
  br i1 %is_zero, label %return, label %addmod

addmod:
  %arg1m = urem i256 %arg1, %modulo
  %arg2m = urem i256 %arg2, %modulo
  %res = call {i256, i1} @llvm.uadd.with.overflow.i256(i256 %arg1m, i256 %arg2m)
  %sum = extractvalue {i256, i1} %res, 0
  %obit = extractvalue {i256, i1} %res, 1
  %sum.mod = urem i256 %sum, %modulo
  br i1 %obit, label %overflow, label %return

overflow:
  %mod.inv = xor i256 %modulo, -1
  %sum1 = add i256 %sum, %mod.inv
  %sum.ovf = add i256 %sum1, 1
  br label %return

return:
  %value = phi i256 [0, %entry], [%sum.mod, %addmod], [%sum.ovf, %overflow]
  ret i256 %value
}

declare {i256, i1} @llvm.uadd.with.overflow.i256(i256, i256)
declare void @llvm.evm.return(ptr addrspace(1), i256)
declare i256 @llvm.evm.calldataload(ptr addrspace(2))

attributes #0 = { mustprogress nofree norecurse nosync nounwind readnone willreturn }
