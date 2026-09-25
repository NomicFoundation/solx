object "ErrorSwitchNoCaseDefault" {
    code {
        {
            switch 42
                branch x {}
                default {}
        }
    }
    object "ErrorSwitchNoCaseDefault_deployed" {
        code {
            { return(0, 0) }
        }
    }
}
