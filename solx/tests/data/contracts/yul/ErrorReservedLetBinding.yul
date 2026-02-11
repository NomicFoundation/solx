object "ErrorReservedLetBinding" {
    code {
        {
            let basefee := 42
            return(0, 0)
        }
    }
    object "ErrorReservedLetBinding_deployed" {
        code {
            { return(0, 0) }
        }
    }
}
