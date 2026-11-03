//! Wire format between the browser and the server. Everything is JSON with a
//! `type` discriminator so the client can tell messages apart.

use serde::{Deserialize, Serialize};

use crate::cipher::Code;

/// Browser -> server.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    /// Turn one cylinder so `index` is the letter facing the solver.
    Rotate { ring: usize, index: u8 },
    /// Submit whatever the rings currently show.
    Attempt,
    /// Admin-only: start a new puzzle. Requires `CRYPTEX_ADMIN_TOKEN`.
    Reset { token: String },
}

/// Values handed to the victory message on the client.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolvedValues {
    pub password: String,
    pub coded: String,
    pub attempts: u32,
    pub solved_at: String,
}

/// Server -> browser.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    /// Full snapshot. Sent on connect and after every change.
    State {
        coded: String,
        rings: Code,
        attempts: u32,
        solved: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        values: Option<SolvedValues>,
    },
    /// Emitted exactly once, the moment the puzzle is opened.
    Solved { values: SolvedValues },
    /// Something the client sent was rejected.
    Error { message: String },
}
