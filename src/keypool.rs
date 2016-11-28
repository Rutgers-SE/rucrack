use util::u8_vector;
use overflow::{WrappedStep, WrappedInc};
use std::fmt;
use rustc_serialize::json::{Json, ToJson};
use std::collections::BTreeMap;
use std::u8;


#[derive(Debug, Clone, RustcDecodable)]
pub struct KeyPool {
    pub dynamic_ms_cap: u8,
    pub dynamic_ms_start: u8,
    pub static_ms_bytes: Vec<u8>,
    pub dynamic_bytes: Vec<u8>,
    pub static_ls_bytes: Vec<u8>,
}

impl KeyPool {
    pub fn new(ms_bytes: u8, dy_bytes: u8, ls_bytes: u8) -> KeyPool {
        // let cap = 256 / parition_count;
        // println!("{} {}", cap, parition_count);
        KeyPool {
            dynamic_ms_cap: 0, // meaning the value will eventuall wrap around
            dynamic_ms_start: 0,
            static_ms_bytes: u8_vector(ms_bytes),
            dynamic_bytes: u8_vector(dy_bytes),
            static_ls_bytes: u8_vector(ls_bytes),
        }
    }

    pub fn split_key(&self, parition_count: u64) -> Vec<KeyPool> {
        let mut output = vec![];

        let mut dynamic_bytes = self.dynamic_bytes.clone();
        let step = if self.dynamic_ms_cap == 0 {
            let o = ((256 - self.dynamic_bytes[0] as u64) / parition_count) as u8;
            // println!("o {}", o);
            o
        } else {
            ((self.dynamic_ms_cap as u64 + 1) / parition_count) as u8
        };

        for _ in 0..parition_count {
            let db = dynamic_bytes.clone();
            let cap = dynamic_bytes[0].step(&step);

            output.push(KeyPool {
                dynamic_ms_cap: cap,
                dynamic_ms_start: db[0],
                static_ms_bytes: self.static_ms_bytes.clone(),
                dynamic_bytes: db,
                static_ls_bytes: self.static_ls_bytes.clone()
            });

            dynamic_bytes[0] = dynamic_bytes[0].step(&step);
        }


        output
    }

    pub fn is_cap_reached(&self) -> bool {
        self.dynamic_bytes[0] == self.dynamic_ms_cap
    }

    pub fn is_done(&self) -> bool {
        let mut done = true;
        for idx in 1..self.dynamic_bytes.len() {
            if self.dynamic_bytes[idx] != 255 {
                done = false;
                break;
            }
        }

        if done && self.is_cap_reached() {
            true
        } else {
            false
        }
    }

    #[allow(unused)]
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

    /// Increments the dynamic bytes to emulate counting
    pub fn inc(&self) -> KeyPool {
        let mut slf = self.clone();
        let mut idx = self.dynamic_bytes.len() - 1;
        loop {
            // bounds checking
            if slf.dynamic_bytes.len() == 0 || idx > slf.dynamic_bytes.len() - 1 {
                break;
            }

            let old_value = slf.dynamic_bytes[idx];
            slf.dynamic_bytes[idx] = slf.dynamic_bytes[idx].inc();
            let progress = old_value > slf.dynamic_bytes[idx];
            if progress && idx != 0 {
                idx -= 1;
            } else {
                break;
            }
        }

        slf.clone()
    }
}

impl fmt::Display for KeyPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "({},{}:{:?})",
               self.dynamic_ms_cap,
               self.dynamic_ms_start,
               self.dynamic_bytes)
    }
}

impl Iterator for KeyPool {
    type Item = KeyPool;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_done() {
            Some(self.inc())
        } else {
            None
        }
    }
}

impl ToJson for KeyPool {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        d.insert("dynamic_ms_cap".to_string(), self.dynamic_ms_cap.to_json());
        d.insert("dynamic_ms_start".to_string(), self.dynamic_ms_start.to_json());
        d.insert("static_ms_bytes".to_string(),
                 self.static_ms_bytes.to_json());
        d.insert("dynamic_bytes".to_string(), self.dynamic_bytes.to_json());
        d.insert("static_ls_bytes".to_string(),
                 self.static_ls_bytes.to_json());

        Json::Object(d)
    }
}

#[test]
fn test_split_key() {
    let mut key = KeyPool::new(14, 2, 0);
    let mut keys = key.split_key(2);

    println!("{:?}", keys);
    assert!(keys[0].dynamic_bytes[0] == 0);
    assert!(keys[0].dynamic_ms_cap == 128);
    assert!(keys[1].dynamic_bytes[0] == 128);
    assert!(keys[1].dynamic_ms_cap == 0);

    key = keys[0].clone();
    keys = key.split_key(2);

    assert!(keys[0].dynamic_bytes[0] == 0);
    assert!(keys[0].dynamic_ms_cap == 64);
    assert!(keys[1].dynamic_bytes[0] == 64);
    assert!(keys[1].dynamic_ms_cap == 128);
}
