use crate::state::SharedState;
use log::{info, debug, error};

// Byte constants based on the LSS script provided (for binary fallback)
const GROUP_INITIALIZE: u8 = 0x10;
const GROUP_TIME: u8 = 0x11;
const GROUP_WIND: u8 = 0x12;
const GROUP_RESULTS_HEADER: u8 = 0x13;
const GROUP_RESULT: u8 = 0x14;
const GROUP_MESSAGE_HEADER: u8 = 0x15;
const GROUP_MESSAGE: u8 = 0x16;
const GROUP_BREAK_TIME: u8 = 0x17;
const GROUP_BREAK_NAME: u8 = 0x18;

// Field Variables
const VAR_RESULT_PLACE: u8 = 0x01;
const VAR_RESULT_LANE: u8 = 0x02;
const VAR_RESULT_ID: u8 = 0x03;
const VAR_RESULT_NAME: u8 = 0x04;
const VAR_RESULT_AFFILIATION: u8 = 0x05;
const VAR_RESULT_TIME: u8 = 0x06;
const VAR_RESULT_DELTA: u8 = 0x07;

pub struct LynxParser {
    state: SharedState,
    buffer: Vec<u8>,
}

impl LynxParser {
    pub fn new(state: SharedState) -> Self {
        Self {
            state,
            buffer: Vec::new(),
        }
    }

    pub fn process_chunk(&mut self, chunk: &[u8]) {
        self.buffer.extend_from_slice(chunk);
        
        if self.buffer.is_empty() { return; }

        let first_byte = self.buffer[0];

        // 1. LSS Binary (Starts with Group Code 0x10 - 0x18)
        if first_byte >= 0x10 && first_byte <= 0x18 {
             self.process_binary_lss();
             return;
        }

        // 2. Message Signature check (0x01 'M' 0x02)
        // Log showed: [01, 4D, 02 ...]
        if let Some(start_idx) = self.find_sequence(&[0x01, 0x4D, 0x02]) {
             self.process_message_signature(start_idx);
             return; 
        }

        // 3. UTF-16LE CSV: Contains null bytes (0x00)
        let has_null = self.buffer.iter().any(|&b| b == 0);
        let has_semi = self.buffer.iter().any(|&b| b == b';');
        let has_comma = self.buffer.iter().any(|&b| b == b',');
        let has_colon = self.buffer.iter().any(|&b| b == b':'); 

        if has_null {
             self.process_csv_utf16();
        } else {
             // ASCII handling
             if has_semi || has_comma {
                 self.process_csv_ascii();
             } else if has_colon {
                 self.process_ascii_time();
             } else {
                 if self.buffer.len() > 100 {
                     self.buffer.clear();
                 }
             }
        }
    }

    fn process_ascii_time(&mut self) {
        // "  12:16:03.0  "
        // Or "*start12:16:03.0"
        if let Ok(s) = String::from_utf8(self.buffer.clone()) {
            let parts: Vec<&str> = s.split_whitespace().collect();
            
            // Check for Gun Start
            let gun_start_part = parts.iter().find(|p| p.contains("*start"));
            if let Some(part) = gun_start_part {
                // "*start10:00:00.0" -> extract "10:00:00.0"
                let time_str = part.replace("*start", "");
                if !time_str.is_empty() {
                    debug!("Parsed Gun Time: '{}'", time_str);
                    let mut state = self.state.write();
                    state.gun_time = time_str.clone();
                    // Also update history if we have an event number
                     if !state.event_number.is_empty() {
                         let key = state.event_number.clone();
                         state.races.entry(key)
                             .and_modify(|race| race.gun_time = time_str.clone());
                     }
                }
            }

            // Standard Running Time
            // Find the last part that looks like a time (contains ':') AND isn't the gun start packet itself if it was mixed?
            // Usually *start comes alone.
            // But if we have "*start10..." standard time parsing might trip or just ignore it if we are careful.
            
            let last_valid_time = parts.iter().rfind(|p| p.contains(':') && !p.contains(',') && !p.contains(';') && !p.contains("*start"));
            
            if let Some(time_str) = last_valid_time {
                 debug!("Parsed ASCII Time: '{}'", time_str);
                 let mut state = self.state.write();
                 state.time = time_str.to_string();
                 state.running = true;
            }
            // Clear buffer after processing
            self.buffer.clear();
        }
    }
    
    fn process_csv_ascii(&mut self) {
        if let Ok(s) = String::from_utf8(self.buffer.clone()) {
             // Process up to last semicolon
             if let Some(last_semi) = s.rfind(';') {
                 let (processed_str, _) = s.split_at(last_semi + 1);
                 self.parse_csv_string(processed_str);
                 
                 // Remove processed bytes
                 // Since ASCII is 1 byte per char, index matches
                 let bytes_to_remove = last_semi + 1;
                 self.buffer.drain(0..bytes_to_remove);
             }
        }
    }

    fn process_csv_utf16(&mut self) {
        let safe_len = self.buffer.len() & !1; 
        if safe_len == 0 { return; }
        
        // Sanity check: If we detected nulls but it's mostly random, we might be parsing garbage.
        // But let's try decoding.
        
        let (valid_bytes, _) = self.buffer.split_at(safe_len);
        
        let u16_vec: Vec<u16> = valid_bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
            
        let full_text = String::from_utf16_lossy(&u16_vec);
        
        if let Some(last_semi) = full_text.rfind(';') {
            let (processed_str, _) = full_text.split_at(last_semi + 1);
            
            self.parse_csv_string(processed_str);
            
            // Calculate bytes to remove
             let mut cut_index = 0;
            for (i, &c) in u16_vec.iter().enumerate().rev() {
                if c == 0x003B { // ';'
                    cut_index = i + 1;
                    break;
                }
            }
            if cut_index > 0 {
                let bytes_to_remove = cut_index * 2;
                self.buffer.drain(0..bytes_to_remove);
            }
        }
    }
    
    fn parse_csv_string(&mut self, text: &str) {
        let records: Vec<&str> = text.split(';').collect();

        if text.contains("Command=LayoutDraw") {
            debug!("LayoutDraw command detected. Clearing results.");
            self.state.write().results.clear();
        }

        for record in records {
            let clean = record.trim();
            if clean.is_empty() { continue; }
            
            // "UNOFFICIAL,15B Group 1 1500m 111m,nwi,15B,1,01,15B-1-01,AUTO,7"
            if clean.starts_with("OFFICIAL") || clean.starts_with("UNOFFICIAL") {
                debug!("Detected Header: {}", clean);
                let mut s = self.state.write();
                s.results.clear(); // Start of new list
                
                // Parse header fields
                let params: Vec<&str> = clean.split(',').collect();
                // 0: Status (UNOFFICIAL)
                // 1: Event Name
                // 2: Wind?
                // 3: Event Number (Race Number)
                // 4: Round
                // 5: Heat
                
                if params.len() > 3 {
                    let evt_name = params[1].trim().to_string();
                    let evt_num = params[3].trim().to_string();
                    debug!("Parsed Event Info - Name: '{}', Num: '{}'", evt_name, evt_num);
                    s.event_name = evt_name;
                    s.event_number = evt_num;
                }
                
                continue;
            }
            
            // Fields: Place, Lane, ID, Name, Affiliation, Time
            let fields: Vec<&str> = clean.split(',').collect();
            let fields_clean: Vec<String> = fields.iter().map(|s| s.trim().to_string()).collect();
            
            if fields_clean.is_empty() { continue; }

            // Result Heuristic: 
            // 1. Starts with Number (Place)
            // 2. OR Has valid Lane OR Has valid ID
            let could_be_result = fields_clean.len() >= 4 && (
                fields_clean[0].parse::<u32>().is_ok() || 
                !fields_clean[1].is_empty() || 
                !fields_clean[2].is_empty()
            );
            
            if could_be_result {
                 let mut res = crate::state::AthleteResult::default();
                 
                 // Map based on observed CSV:
                 // 0: Place
                 // 1: Lane
                 // 2: ID
                 // 3: Name
                 // 4: Affiliation
                 // 5: Time
                 
                 if fields_clean.len() > 0 { res.place = fields_clean[0].clone(); }
                 if fields_clean.len() > 1 { res.lane = fields_clean[1].clone(); }
                 if fields_clean.len() > 2 { res.id = fields_clean[2].clone(); }
                 if fields_clean.len() > 3 { res.name = fields_clean[3].clone(); }
                 if fields_clean.len() > 4 { res.affiliation = fields_clean[4].clone(); }
                 if fields_clean.len() > 5 { res.time = fields_clean[5].clone(); }
                 
                 debug!("Parsed Athlete: {} (Place: {}, Time: {})", res.name, res.place, res.time);
                 
                 let mut s = self.state.write();
                 
                 // Upsert into Live Results
                 if let Some(existing_idx) = s.results.iter().position(|r| r.lane == res.lane && !r.lane.is_empty()) {
                     s.results[existing_idx] = res.clone();
                 } else {
                     s.results.push(res.clone());
                 }
                 
                 // Update History
                 if !s.event_number.is_empty() {
                     let key = s.event_number.clone();
                     let evt_name = s.event_name.clone();
                     let current_gun_time = s.gun_time.clone();
                     
                     s.races.entry(key.clone())
                         .and_modify(|race| {
                             if let Some(existing_idx) = race.results.iter().position(|r| r.lane == res.lane && !r.lane.is_empty()) {
                                 race.results[existing_idx] = res.clone();
                             } else {
                                 race.results.push(res.clone());
                             }
                         })
                         .or_insert_with(|| {
                             crate::state::RaceData {
                                 event_name: evt_name,
                                 event_number: key,
                                 gun_time: current_gun_time,
                                 results: vec![res],
                             }
                         });
                         
                     // Persist changes
                     crate::state::save_history(&s.races);
                 }
            }
        }
    }

    fn process_binary_lss(&mut self) {
        // Legacy fallback
        loop {
            if self.buffer.is_empty() { break; }
            let group_code = self.buffer[0];
            let mut consumed = 0;

            match group_code {
                GROUP_TIME => {
                    if let Some(end_idx) = self.find_sequence(&[0x03, 0x04]) {
                        if let Some(start_idx) = self.find_sequence(&[0x02]) {
                             if start_idx < end_idx {
                                 let time_bytes = &self.buffer[start_idx+1..end_idx];
                                 let time_str = String::from_utf8_lossy(time_bytes).trim().to_string();
                                 debug!("Recognized LSS TIME packet: '{}'", time_str);
                                 let mut s = self.state.write();
                                 s.time = time_str;
                                 s.running = true;
                             }
                        }
                        consumed = end_idx + 2;
                    }
                },
                GROUP_MESSAGE_HEADER => {
                    // Header: \15\00\01M\02
                    // Trailer: \15\00\03\04
                    // We can just detect \15 and try to find the end byte.
                    // For header, end is \02? For trailer \04?
                    // Simplified: just consume until \02 or \04.
                    
                    if let Some(end_idx) = self.find_sequence(&[0x02]) {
                         debug!("Detected LSS Message Header. Clearing old messages.");
                         self.state.write().messages.clear();
                         consumed = end_idx + 1;
                    } else if let Some(end_idx) = self.find_sequence(&[0x03, 0x04]) {
                         // Trailer
                         consumed = end_idx + 2;
                    }
                },
                GROUP_MESSAGE => {
                    // \16 \01 [Text] \05
                    if let Some(end_idx) = self.find_sequence(&[0x05]) {
                         // Check for \01 at start of value
                         // buffer[0] is 0x16. buffer[1] should be 0x01.
                         if self.buffer.len() > 2 && self.buffer[1] == 0x01 {
                             let val_bytes = &self.buffer[2..end_idx];
                             let val = String::from_utf8_lossy(val_bytes).to_string(); // Don't trim to preserve spacing if needed?
                             debug!("Parsed LSS Message: '{}'", val);
                             self.state.write().messages.push(val);
                         }
                         consumed = end_idx + 1;
                    }
                },
                _ => { consumed = 1; }
            }
            if consumed > 0 { self.buffer.drain(0..consumed); } else { break; }
        }
    }

    fn process_message_signature(&mut self, start_idx: usize) {
        let header_end = start_idx + 3;
        
        // Trailer: \03\04
        if let Some(trailer_idx) = self.find_sequence(&[0x03, 0x04]) {
             if trailer_idx >= header_end {
                 debug!("Detected Message Signature. Clearing old messages.");
                 self.state.write().messages.clear();
                 
                 let content_slice = &self.buffer[header_end..trailer_idx];
                 let content_len = content_slice.len();
                 
                 if content_len > 0 {
                     let text_slice = if content_slice[content_len - 1] == 0x05 {
                         &content_slice[..content_len - 1]
                     } else {
                         content_slice
                     };
                     
                     if !text_slice.is_empty() {
                         let msg = String::from_utf8_lossy(text_slice).to_string();
                         debug!("Parsed Message Text: '{}'", msg);
                         self.state.write().messages.push(msg);
                     }
                 }
                 
                 let consumed = trailer_idx + 2;
                 self.buffer.drain(0..consumed);
             }
        }
    }

    fn find_sequence(&self, seq: &[u8]) -> Option<usize> {
        self.buffer.windows(seq.len()).position(|window| window == seq)
    }
    
    fn handle_result_field(&mut self, _var_code: u8, _value: String) {
        // Legacy
    }
}
