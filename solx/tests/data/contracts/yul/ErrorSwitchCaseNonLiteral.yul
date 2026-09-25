object "ErrorSwitchCaseNonLiteral" {
    code {
        {
            switch 42
                case x {}
                default {}
        }
    }
    object "ErrorSwitchCaseNonLiteral_deployed" {
        code {
            { return(0, 0) }
        }
    }
}
