object "ErrorUnsupportedSelfdestruct" {
    code {
        {
            let size := datasize("ErrorUnsupportedSelfdestruct_deployed")
            codecopy(0, dataoffset("ErrorUnsupportedSelfdestruct_deployed"), size)
            return(0, size)
        }
    }
    object "ErrorUnsupportedSelfdestruct_deployed" {
        code {
            {
                selfdestruct(caller())
            }
        }
    }
}
