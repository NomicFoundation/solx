object "Selfdestruct" {
    code {
        {
            let size := datasize("Selfdestruct_deployed")
            codecopy(0, dataoffset("Selfdestruct_deployed"), size)
            return(0, size)
        }
    }
    object "Selfdestruct_deployed" {
        code {
            {
                selfdestruct(caller())
            }
        }
    }
}
