object "ParserCoverage" {
    code {
        {
            let size := datasize("ParserCoverage_deployed")
            codecopy(0, dataoffset("ParserCoverage_deployed"), size)
            return(0, size)
        }
    }
    object "ParserCoverage_deployed" {
        code {
            {
                let a := 42
                let b := 0xff
                let c
                c := add(a, b)
                let d, e, f := multiReturn()
                if gt(c, 0) { mstore(0, c) }
                switch a
                case 42 { mstore(0, 1) }
                case 0 { mstore(0, 2) }
                default { mstore(0, 3) }
                for { let i := 0 } lt(i, 10) { i := add(i, 1) } {
                    if eq(i, 5) { break }
                    if eq(i, 3) { continue }
                    mstore(0, i)
                }
                if true { mstore(0, 1) }
                { { let nested := 99 mstore(0, nested) } }
                d, e, f := multiReturn()
                mstore(0, d)
                return(0, 32)
                function helper() -> result {
                    result := 1
                    leave
                }
                function multiReturn() -> r1, r2, r3 {
                    r1 := 1
                    r2 := 2
                    r3 := 3
                }
            }
        }
    }
}
