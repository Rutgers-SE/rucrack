use std::fs::File;
use std::env::args;
use std::io::prelude::*;
use std::path::Path;
use std::io::stdout;

// impl IntoUrl for SocketAddr {
//
// }

// for now this only will work with kde neon
pub fn load_linux_dictionary() -> Option<Vec<String>> {
    match File::open("/etc/dictionaries-common/words") {
        Ok(mut file) => {
            let mut output: Vec<String> = vec![];
            let mut buffer = String::new();
            match file.read_to_string(&mut buffer) {
                Ok(_) => {
                    Some(buffer.split("\n")
                        .map(|x| x.to_string())
                        .collect())
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

pub fn u8_vector(amount: u8) -> Vec<u8> {
    let mut v: Vec<u8> = vec![];

    for i in 0..amount {
        v.push(0);
    }

    v
}


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

// remember to add some stuff here...
// we could use the english dictionary supplied with every linux distro....
// maybe different for mac's and pc's
pub fn is_english(plain_text: String) -> bool {
    true
}

pub fn prompt(s: &str) {
    print!("{}", s);
    stdout().flush();
}
