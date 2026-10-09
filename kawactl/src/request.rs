use serde::{Deserialize,Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "cmd", rename_all = "kebab-case")]
pub enum Request {
    Version,
    Get{
        what: GetReq
    },
    Close,
    Quit,
    Exit,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "get", rename_all = "kebab-case")]
pub enum GetReq{
    Outputs,
    Windows
}