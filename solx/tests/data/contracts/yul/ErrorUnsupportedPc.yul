object "ErrorUnsupportedPc" {
    code {
        {
            let size := datasize("ErrorUnsupportedPc_deployed")
            codecopy(0, dataoffset("ErrorUnsupportedPc_deployed"), size)
            return(0, size)
        }
    }
    object "ErrorUnsupportedPc_deployed" {
        code {
            {
                let x := pc()
                return(0, 0)
            }
        }
    }
}
