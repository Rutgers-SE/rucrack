use util::u8_vector;
use overflow::{WrappedStep, WrappedInc};
use std::fmt;
use rustc_serialize::json::{Json, ToJson};
use std::collections::BTreeMap;


#[derive(Debug, Clone, RustcDecodable)]
pub struct KeyPool {
    pub dynamic_ms_cap: u8,
    pub parition_count: u8,
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
            parition_count: 1,
            static_ms_bytes: u8_vector(ms_bytes),
            dynamic_bytes: u8_vector(dy_bytes),
            static_ls_bytes: u8_vector(ls_bytes),
        }
    }

    pub fn split_key(&self, parition_count: i64) -> Vec<KeyPool> {
        if parition_count <= 0 {
            return vec![self.clone()];
        }
        let mut output = vec![];
        let kp: KeyPool = self.clone();
        let step = if kp.dynamic_ms_cap == 0 {
            ((256) / parition_count) as u8
        } else {
            ((kp.dynamic_ms_cap as i64 + 1) / parition_count) as u8
        };
        println!("{}", step);
        let mut cap = step as u8;
        let mut cursor = kp.dynamic_bytes[0];

        for _ in 0..parition_count {
            let mut db = u8_vector(kp.dynamic_bytes.len() as u8);
            db[0] = cursor;
            // println!("{:?}, {:?}", cap, cursor);

            output.push(KeyPool {
                dynamic_ms_cap: cap.clone(),
                parition_count: parition_count as u8,
                static_ms_bytes: kp.static_ms_bytes.clone(),
                dynamic_bytes: db,
                static_ls_bytes: kp.static_ls_bytes.clone(),
            });

            cursor = cursor.step(&(step as u8));
            cap = cap.step(&(step as u8));
            // println!("{:?}, {:?}, {:?}", cap, cursor, step);
        }

        output
    }

    pub fn is_cap_reached(&self) -> bool {
        self.dynamic_bytes[0] == self.dynamic_ms_cap
    }

    pub fn is_done(&self) -> bool {
        let mut done = true;
        if self.parition_count != 1 {
            if !self.is_cap_reached() {
                done = false;
            }
        } else {
            if self.dynamic_bytes[0] != 255 {
                done = false;
            }
        }
        for idx in 1..self.dynamic_bytes.len() {
            if self.dynamic_bytes[idx] != 255 {
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
            if progress && idx != 0 {
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
        write!(f,
               "({},{}:{:?})",
               self.dynamic_ms_cap,
               self.parition_count,
               self.to_vec())
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
        d.insert("parition_count".to_string(), self.parition_count.to_json());
        d.insert("static_ms_bytes".to_string(),
                 self.static_ms_bytes.to_json());
        d.insert("dynamic_bytes".to_string(), self.dynamic_bytes.to_json());
        d.insert("static_ls_bytes".to_string(),
                 self.static_ls_bytes.to_json());

        Json::Object(d)
    }
}

#[test]
fn test_generate_keys() {
    let keys = KeyPool::generate_keys(2, 2 as u8);

    assert!(keys.len() == 2);

    assert!(keys[0].dynamic_bytes[0] == 0);
    assert!(keys[0].dynamic_ms_cap == 128);
    assert!(keys[1].dynamic_bytes[0] == 128);
    assert!(keys[1].dynamic_ms_cap == 0);
}

#[test]
fn test_split_key() {
    let mut key = KeyPool::new(1, 14, 2, 0);
    let mut keys = key.split_key(2);

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
