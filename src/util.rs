use std::fs::File;
use std::io::prelude::*;
use std::io::stdout;
use std::u8;
use std::string::ToString;


pub fn u8_vector(amount: u8) -> Vec<u8> {
    let mut v: Vec<u8> = vec![];

    for _ in 0..amount {
        v.push(0);
    }

    v
}

pub fn read_r4c_file(arg: String) -> Option<Vec<u8>> {
    let mut output: Vec<u8> = vec![];
    let mut file = File::open(arg).unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();

    match buf.pop() { // nice
        Some(c) if c != '\n' => {
            println!("{}", c);
            buf.push(c);
        }
        _ => ()
    }

    let mut count = 0;
    let mut tmp = 0;

    for ch in buf.chars() {
        let s = ch.to_string();
        let u4 = match u8::from_str_radix(&s, 16) {
            Ok(e) => e,
            Err(e) => {
                panic!("I paniced with {:?} -- {:?}", s, e)
            }
        };
        if count % 2 == 0 {
            // output.push(u4);
            tmp = u4 << 4;
        } else {
            let back = tmp | u4;
            output.push(back);
        }
        count = count + 1;
    }


    Some(output)
}

#[allow(unused)]
pub fn read_file_from_arg(arg: Option<String>) -> Option<Vec<u8>> {
    match arg {
        Some(name) => {
            match File::open(name) {
                Ok(mut file) => {
                    let mut output: Vec<u8> = vec![];
                    match file.read_to_end(&mut output) {
                        Ok(_) => Some(output),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }
        None => None,
    }
}

// NOTE: it appears that we do not need this method because of how rust implements strings. (AWESOME)
pub fn is_english(_: String) -> bool {
    true
}

pub fn prompt(s: &str) {
    print!("{}", s);
    stdout().flush().unwrap();
}
