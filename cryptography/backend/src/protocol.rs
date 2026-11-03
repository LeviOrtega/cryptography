use serde::{Deserialize, Serialize};

use crate::cipher::Code;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Rotate { ring: usize, index: u8 },
    Attempt,
    Reset { token: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolvedValues {
    pub password: String,
    pub coded: String,
    pub attempts: u32,
    pub solved_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    State {
        coded: String,
        rings: Code,
        attempts: u32,
        solved: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        values: Option<SolvedValues>,
    },
    Solved { values: SolvedValues },
    Error { message: String },
}
