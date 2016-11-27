use util::{u8_vector, read_file_from_arg, is_english, load_linux_dictionary};
use keypool::KeyPool;
use encryption::{encrypt, decrypt};

use std::sync::mpsc::{channel, Sender};
use std::env::args;
use std::thread;
use std::str::{from_utf8, from_utf8_unchecked};



fn dc(cipher_text: &Vec<u8>, key: &KeyPool, sender: &Sender<Vec<u8>>) {
    let mut key = key.clone();
    let mut invalid_count = 0;
    let mut fail_count = 0;
    loop {
        match decrypt(&cipher_text[..], &key.to_vec()) {
            Ok(decrypted_data) => {
                match from_utf8(&decrypted_data) {
                    Ok(pt) => {
                        println!("Msg found \"{}\"", pt);
                        if is_english(pt.to_string()) {
                            sender.send(key.to_vec());
                        }
                    }
                    Err(e) => {
                        // println!("{}", e);
                        invalid_count = invalid_count + 1;
                        ()
                    }

                }
            }
            Err(e) => {
                // println!("{:?}", e);
                fail_count = fail_count + 1;
                ()
            }
        }
        key = key.inc();
        if key.is_done() {
            break;
        }
    }
    println!("Invalid string {} time(s)\nFailed decode {} times",
        invalid_count,
        fail_count);
}

// returns potential keys
pub fn crack(kp: KeyPool, cipher_text: Vec<u8>, thread_count: i64) -> Vec<Vec<u8>> {
    let keys = kp.split_key(thread_count);

    let mut threads = vec![]; // thread pool

    // com. channel
    let (tx, rx) = channel();
    for mut key in keys {

        // Cloning to make it thread safe
        let tx = tx.clone();
        let cipher_text = cipher_text.clone();

        threads.push(thread::spawn(move || {
            dc(&cipher_text, &key, &tx);
        }));
    }

    for t in threads {
        t.join();
    }
    tx.send(vec![1]);

    let mut output = vec![];

    loop {
        match rx.recv() {
            Ok(vector) => {
                if vector.len() == 1 {
                    break;
                }
                output.push(vector);
            }
            Err(_) => break,
        }
    }

    output
}
