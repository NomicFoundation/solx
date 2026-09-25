object "SelfdestructDeployCode" {
    code {
        {
            selfdestruct(caller())
        }
    }
    object "SelfdestructDeployCode_deployed" {
        code {
            {
                return(0, 0)
            }
        }
    }
}
