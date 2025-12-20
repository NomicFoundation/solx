//! { "modes": [ "Y", "E" ], "cases": [ {
//!     "name": "ascending",
//!     "inputs": [
//!         {
//!             "method": "check_entrypoint",
//!             "calldata": [
//!                 "0x80000000000000000000000000000000f876da3c85160abc0770c91f44774636",
//!                 "0x000000000000000000000000000000003bde33efbf52406aed04e17904f90b64",
//!                 "0x800000000000000000000000000000003ff28daac2801d83ac1d10e9aca5a5a9",
//!                 "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                 "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                 "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                 "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                 "0x0000000000000000000000000000000000000000000000000000000000000000"
//!             ]
//!         }
//!     ],
//!     "expected": []
//! } ] }
// SPDX-License-Identifier: MIT

pragma solidity >=0.8.30;

contract Test {
    function $_4xN7uxIt7qtlv(
        bool assert_in3,
        bool assert_in5,
        bool assert_in7
    ) internal returns (bool) {
        return (assert_in5 ||
            ((true && assert_in7) && (assert_in3 && assert_in7)));
    }

    function w7Z$oKNuwzNJk78RIsb030(
        bool assert_in3,
        bool assert_in4,
        bool assert_in5,
        bool assert_in6,
        bool assert_in7
    ) internal returns (bool) {
        return ((((false ||
            (assert_in6 && (assert_in6 && (assert_in6 && (true || false))))) ||
            (false || assert_in6)) ||
            (false || (assert_in7 || (assert_in7 && true)))) ||
            (((assert_in7 ||
                (assert_in3 ||
                    ((!(!(assert_in3 || assert_in4))) ||
                        (!(!(assert_in4 || assert_in4)))))) &&
                (true || (assert_in3 || assert_in4))) &&
                (((((assert_in3 || (assert_in4 && true)) ||
                    (assert_in3 || assert_in4)) &&
                    ((true || true) || (assert_in4 || assert_in3))) ||
                    (((((assert_in3 || (assert_in4 && true)) ||
                        (assert_in3 || assert_in4)) &&
                        (true || (assert_in4 || assert_in3))) ||
                        (false || false)) &&
                        ((((assert_in3 || (assert_in4 && true)) ||
                            (assert_in3 || assert_in4)) &&
                            (true || (assert_in4 || assert_in3))) ||
                            (false || false)))) || assert_in5)));
    }

    function FQkY3(
        bool assert_in3,
        bool assert_in5,
        bool assert_in7
    ) internal virtual returns (bool) {
        return ((((assert_in7 && true) ||
            ((assert_in7 && (true && (assert_in3 && assert_in7))) ||
                assert_in5)) ||
            ((((assert_in7 && true) ||
                ((assert_in7 && true) && (assert_in3 && assert_in7))) ||
                assert_in5) && true)) &&
            (((assert_in3 ||
                (true && (assert_in7 && (assert_in7 && assert_in3)))) &&
                (assert_in7 ||
                    ((true && true) &&
                        (assert_in7 && (assert_in3 && assert_in7))))) ||
                assert_in5));
    }

    function Nr9GYw_(
        bool assert_in3,
        bool assert_in4,
        bool assert_in5,
        bool assert_in6,
        bool assert_in7
    ) internal returns (bool) {
        return (w7Z$oKNuwzNJk78RIsb030(
            assert_in3,
            assert_in4,
            assert_in5,
            assert_in6,
            assert_in7
        ) || FQkY3(assert_in3, assert_in5, assert_in7));
    }

    function check_entrypoint(
        int256 assert_in0,
        int256 assert_in1,
        int256 assert_in2,
        bool assert_in3,
        bool assert_in4,
        bool assert_in5,
        bool assert_in6,
        bool assert_in7
    ) public {
        unchecked {
            bool assert_out1 = ((((assert_in4 || assert_in3) &&
                (assert_in3 || assert_in4)) ||
                ((assert_in7 && assert_in5) || (assert_in6 || assert_in7))) ||
                $_4xN7uxIt7qtlv(assert_in3, assert_in5, assert_in7));
            bool assert_out2 = Nr9GYw_(
                assert_in3,
                assert_in4,
                assert_in5,
                assert_in6,
                assert_in7
            );
            assert((assert_out1 == assert_out2));
        }
    }
}
