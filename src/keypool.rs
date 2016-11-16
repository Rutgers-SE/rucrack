use util::{u8_vector};
use overflow::{WrappedStep, WrappedInc};
use std::fmt;

#[derive(Debug, Clone)]
pub struct KeyPool {
    dynamic_ms_cap: u8,
    pub parition_count: u8,
    pub static_ms_bytes: Vec<u8>,
    pub dynamic_bytes: Vec<u8>,
    pub static_ls_bytes: Vec<u8>,
}

impl KeyPool {
    pub fn new(parition_count: i64, ms_bytes: u8, dy_bytes: u8, ls_bytes: u8) -> KeyPool {
        let cap = 256 / parition_count;
        println!("{} {}", cap, parition_count);
        KeyPool {
            dynamic_ms_cap: (cap as u8).dec(),
            parition_count: parition_count as u8,
            static_ms_bytes: u8_vector(ms_bytes),
            dynamic_bytes: u8_vector(dy_bytes),
            static_ls_bytes: u8_vector(ls_bytes),
        }
    }

    // returns a vector of keys containing the amount specified by the `parition_count`
    pub fn generate_keys(parition_count: i64, dynamic_byte_len: u8) -> Vec<KeyPool> {
        let mut output: Vec<KeyPool> = vec![];
        let step = 256 / parition_count;
        let mut cap = step as u8;
        let mut cursor = 0;

        for key_id in 0..parition_count {
            let mut db = u8_vector(dynamic_byte_len);
            println!("Cursor: {}", cursor);
            db[0] = cursor;
            output.push(KeyPool {
                dynamic_ms_cap: cap.clone(),
                parition_count: parition_count as u8,
                static_ms_bytes: vec![],
                dynamic_bytes: db,
                static_ls_bytes: vec![]
            });
            cursor = cursor.step(&(step as u8));
            cap = cap.step(&(step as u8));
        }

        output
    }

    pub fn is_done(&self) -> bool {
        let mut done = true;
        for idx in 1..self.dynamic_bytes.len() {
            if self.dynamic_bytes[idx] != 255 || self.dynamic_bytes[0] != self.dynamic_ms_cap {
                done = false;
                break;
            }
        }
        done
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut output: Vec<u8> = vec![];
        for b in self.static_ms_bytes.clone() {
            output.push(b);
        }
        for b in self.dynamic_bytes.clone() {
            output.push(b);
        }
        for b in self.static_ls_bytes.clone() {
            output.push(b);
        }

        output
    }

    /// Increments the dynamic bytes to emulate
    pub fn inc(&mut self) -> KeyPool {
        let mut idx = self.dynamic_bytes.len() - 1;
        loop {
            // bounds checking
            if self.dynamic_bytes.len() == 0 || idx > self.dynamic_bytes.len() - 1 {
                break;
            }

            let tmp = self.dynamic_bytes[idx];
            self.dynamic_bytes[idx] = self.dynamic_bytes[idx].inc();
            let progress = tmp > self.dynamic_bytes[idx];
            if progress {
                idx -= 1;
            } else {
                break;
            }
        }

        KeyPool {
            dynamic_ms_cap: self.dynamic_ms_cap.clone(),
            parition_count: self.parition_count.clone(),
            static_ms_bytes: self.static_ms_bytes.clone(),
            dynamic_bytes: self.dynamic_bytes.clone(),
            static_ls_bytes: self.static_ls_bytes.clone(),
        }

    }
}

impl fmt::Display for KeyPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{}:{:?})",
            self.dynamic_ms_cap,
            self.parition_count,
            self.to_vec())
    }
}
