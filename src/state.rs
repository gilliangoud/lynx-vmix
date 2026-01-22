use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoreboardState {
    pub time: String,
    pub running: bool,
    pub results: Vec<AthleteResult>,
    pub messages: Vec<String>,
    pub event_name: String,
    pub event_number: String,
    pub gun_time: String,
    
    // History of races, keyed by event_number
    pub races: HashMap<String, RaceData>,

    // For intermediate parsing verification or debugging
    pub last_packet: String, 
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RaceData {
    pub event_name: String,
    pub event_number: String,
    pub gun_time: String,
    pub results: Vec<AthleteResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AthleteResult {
    pub place: String,
    pub lane: String,
    pub id: String,
    pub name: String,
    pub affiliation: String,
    pub time: String,
    pub delta_time: String,
    // Add more fields as needed from LSS
}

// We will use Parking Lot RwLock for the global state
use parking_lot::RwLock;
use std::sync::Arc;

use std::fs;
use std::path::Path;

pub type SharedState = Arc<RwLock<ScoreboardState>>;

const HISTORY_FILE: &str = "events-history.json";

pub fn initialize_state() -> SharedState {
    let mut state = ScoreboardState::default();
    state.races = load_history();
    Arc::new(RwLock::new(state))
}

pub fn load_history() -> HashMap<String, RaceData> {
    if Path::new(HISTORY_FILE).exists() {
        if let Ok(content) = fs::read_to_string(HISTORY_FILE) {
            if let Ok(data) = serde_json::from_str(&content) {
                println!("Loaded history from {}", HISTORY_FILE);
                return data;
            }
        }
    }
    HashMap::new()
}

pub fn save_history(races: &HashMap<String, RaceData>) {
    if let Ok(json) = serde_json::to_string_pretty(races) {
        if let Err(e) = fs::write(HISTORY_FILE, json) {
            eprintln!("Failed to save history: {}", e);
        }
    }
}
