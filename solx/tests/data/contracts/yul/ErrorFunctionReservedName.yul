object "ErrorFunctionReservedName" {
    code {
        {
            return(0, 0)
        }
    }
    object "ErrorFunctionReservedName_deployed" {
        code {
            {
                return(0, 0)
            }

            function basefee() -> result {
                result := 42
            }
        }
    }
}
