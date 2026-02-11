object "ErrorFunctionBadReturn" {
    code {
        {
            return(0, 0)
        }
    }
    object "ErrorFunctionBadReturn_deployed" {
        code {
            {
                return(0, 0)
            }

            function test() := result {
                result := 42
            }
        }
    }
}
