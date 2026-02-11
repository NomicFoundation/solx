object "ErrorFunctionLiteralName" {
    code {
        {
            return(0, 0)
        }
    }
    object "ErrorFunctionLiteralName_deployed" {
        code {
            {
                return(0, 0)
            }

            function 256() -> result {
                result := 42
            }
        }
    }
}
