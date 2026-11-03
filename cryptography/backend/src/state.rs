use std::sync::Arc;

use tokio::sync::{RwLock, broadcast};

use crate::cipher::{self, Code, RINGS};
use crate::protocol::{ServerMsg, SolvedValues};

pub struct Game {
    key: Code,
    password: Code,
    coded: Code,
    rings: Code,
    attempts: u32,
    solved: Option<SolvedValues>,
}

pub enum AttemptOutcome {
    Solved(SolvedValues),
    Shifted,
    AlreadySolved,
}

impl Game {
    pub fn new(key: Code) -> Self {
        let password = cipher::random_password();
        let coded = cipher::derive_coded(&password, &key);
        tracing::info!(
            password = %cipher::encode(&password),
            coded = %cipher::encode(&coded),
            "new puzzle"
        );
        Self {
            key,
            password,
            coded,
            rings: [0; RINGS],
            attempts: 0,
            solved: None,
        }
    }

    pub fn snapshot(&self) -> ServerMsg {
        ServerMsg::State {
            coded: cipher::encode(&self.coded),
            rings: self.rings,
            attempts: self.attempts,
            solved: self.solved.is_some(),
            values: self.solved.clone(),
        }
    }

    pub fn rotate(&mut self, ring: usize, index: u8) -> Result<bool, String> {
        if self.solved.is_some() {
            return Ok(false);
        }
        if ring >= RINGS {
            return Err(format!("ring {ring} out of range"));
        }
        if index as i32 >= cipher::ALPHABET_LEN {
            return Err(format!("index {index} out of range"));
        }
        if self.rings[ring] == index {
            return Ok(false);
        }
        self.rings[ring] = index;
        tracing::debug!(ring, index, rings = ?self.rings, "rotate");
        Ok(true)
    }

    pub fn attempt(&mut self) -> AttemptOutcome {
        if self.solved.is_some() {
            return AttemptOutcome::AlreadySolved;
        }
        self.attempts += 1;

        if self.rings == self.password {
            let values = SolvedValues {
                password: cipher::encode(&self.password),
                coded: cipher::encode(&self.coded),
                attempts: self.attempts,
                solved_at: chrono::Utc::now().to_rfc3339(),
            };
            self.solved = Some(values.clone());
            tracing::info!(attempts = self.attempts, "puzzle solved");
            return AttemptOutcome::Solved(values);
        }

        self.password = cipher::random_password();
        self.coded = cipher::derive_coded(&self.password, &self.key);
        tracing::info!(
            attempts = self.attempts,
            password = %cipher::encode(&self.password),
            coded = %cipher::encode(&self.coded),
            "wrong attempt, message shifted"
        );
        AttemptOutcome::Shifted
    }

    pub fn reset(&mut self) {
        *self = Game::new(self.key);
    }
}

pub struct AppState {
    pub game: RwLock<Game>,
    pub tx: broadcast::Sender<ServerMsg>,
    pub admin_token: Option<String>,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub fn new(key: Code, admin_token: Option<String>) -> SharedState {
        let (tx, _rx) = broadcast::channel(256);
        Arc::new(Self {
            game: RwLock::new(Game::new(key)),
            tx,
            admin_token,
        })
    }

    pub fn broadcast(&self, msg: ServerMsg) {
        let _ = self.tx.send(msg);
    }
}
