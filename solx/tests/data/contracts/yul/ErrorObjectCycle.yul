object "ErrorObjectCycle" {
    code {
        {
            sstore(0, dataoffset("ErrorObjectCycle_deployed"))
            return(0, 0)
        }
    }
    object "ErrorObjectCycle_deployed" {
        code {
            {
                sstore(0, dataoffset("ErrorObjectCycle"))
                return(0, 0)
            }
        }
    }
}
