object "Simple" {
    code {
        datacopy(0, dataoffset("Simple_deployed"), datasize("Simple_deployed"))
        return(0, datasize("Simple_deployed"))
    }
    object "Simple_deployed" {
        code {
            mstore(0, 42)
            return(0, 32)
        }
    }
}
