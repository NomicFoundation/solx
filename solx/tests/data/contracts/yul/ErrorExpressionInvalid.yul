object "ErrorExpressionInvalid" {
    code {
        {
            if := { }
        }
    }
    object "ErrorExpressionInvalid_deployed" {
        code {
            { return(0, 0) }
        }
    }
}
