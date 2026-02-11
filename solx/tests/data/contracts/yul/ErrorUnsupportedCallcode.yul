object "ErrorUnsupportedCallcode" {
    code {
        {
            let size := datasize("ErrorUnsupportedCallcode_deployed")
            codecopy(0, dataoffset("ErrorUnsupportedCallcode_deployed"), size)
            return(0, size)
        }
    }
    object "ErrorUnsupportedCallcode_deployed" {
        code {
            {
                let success := callcode(gas(), 0, 0, 0, 0, 0, 0)
                return(0, 0)
            }
        }
    }
}
