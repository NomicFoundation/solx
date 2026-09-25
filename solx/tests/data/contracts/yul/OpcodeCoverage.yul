object "OpcodeCoverage" {
    code {
        {
            let size := datasize("OpcodeCoverage_deployed")
            codecopy(0, dataoffset("OpcodeCoverage_deployed"), size)
            return(0, size)
        }
    }
    object "OpcodeCoverage_deployed" {
        code {
            {
                // Arithmetic: sdiv, smod, addmod, mulmod, exp, signextend
                let a := sdiv(10, 3)
                let b := smod(10, 3)
                let c := addmod(10, 20, 7)
                let d := mulmod(10, 20, 7)
                let e := exp(2, 8)
                let f := signextend(0, 0xff)

                // Comparison: slt, sgt, iszero
                let g := slt(1, 2)
                let h := sgt(2, 1)
                let j := iszero(0)

                // Bitwise: or, xor, and, not, shl, shr, sar, byte
                let k := or(0xf0, 0x0f)
                let l := xor(0xff, 0x0f)
                let m := and(0xff, 0x0f)
                let n := not(0)
                let o := shl(1, 0x01)
                let p := shr(1, 0x10)
                let q := sar(1, 0x80000000)
                let r := byte(31, 0xff)

                // Pop
                pop(add(1, 2))

                // Memory: mstore8, mload (mstore already in ParserCoverage)
                mstore8(0, 0x42)
                let mem := mload(0)

                // Keccak256
                mstore(0, 42)
                let hash := keccak256(0, 32)

                // Storage: sload, sstore
                sstore(0, 42)
                let stored := sload(0)

                // Transient storage: tload, tstore
                tstore(0, 99)
                let tval := tload(0)

                // Calldata: calldataload, calldatasize, calldatacopy
                let cdval := calldataload(0)
                let cdsz := calldatasize()
                calldatacopy(0, 0, cdsz)

                // Return data: returndatasize, returndatacopy
                let rdsz := returndatasize()

                // Code: codesize, codecopy
                let csz := codesize()
                codecopy(0x100, 0, 32)

                // Contract context: address, caller, callvalue, gas, balance, selfbalance
                let addr := address()
                let clr := caller()
                let cv := callvalue()
                let g2 := gas()
                let bal := balance(addr)
                let sbal := selfbalance()

                // Block/tx context: gaslimit, gasprice, origin, chainid, timestamp, number,
                //   blockhash, difficulty, coinbase, basefee, msize
                let gl := gaslimit()
                let gp := gasprice()
                let orig := origin()
                let cid := chainid()
                let ts := timestamp()
                let bn := number()
                let bh := blockhash(sub(bn, 1))
                let diff := difficulty()
                let cb := coinbase()
                let bf := basefee()
                let msz := msize()

                // Extcode: extcodesize, extcodehash
                let esz := extcodesize(addr)
                let ehash := extcodehash(addr)

                // Events: log0, log1, log2, log3, log4
                mstore(0, 1)
                log0(0, 32)
                log1(0, 32, 0xaa)
                log2(0, 32, 0xaa, 0xbb)
                log3(0, 32, 0xaa, 0xbb, 0xcc)
                log4(0, 32, 0xaa, 0xbb, 0xcc, 0xdd)

                // Memory copy: mcopy
                mstore(0, 0x1234)
                mcopy(0x20, 0, 32)

                // Switch with only default (no cases)
                switch 1
                default { mstore(0, 0) }

                // Multi-binding without expression
                let x, y, z
                x, y, z := tripleReturn()

                // Return
                mstore(0, stored)
                return(0, 32)

                function tripleReturn() -> r1, r2, r3 {
                    r1 := 10
                    r2 := 20
                    r3 := 30
                }
            }
        }
    }
}
