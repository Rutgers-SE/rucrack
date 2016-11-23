use util::{u8_vector, read_file_from_arg, is_english, load_linux_dictionary};
use keypool::KeyPool;
use encryption::{encrypt, decrypt};

use std::sync::mpsc::{channel, Sender};
use std::env::args;
use std::thread;
use std::str::{from_utf8};



fn dc(cipher_text: &Vec<u8>, key: &KeyPool, sender: &Sender<Vec<u8>>) {
    // let mut output = vec![];
    let mut key = key.clone();
    loop {
        match decrypt(&cipher_text[..], &key.to_vec()) {
            Ok(decrypted_data) => {
                match from_utf8(&decrypted_data) {
                    Ok(pt) => {
                        println!("Found Key  {:?}", key.to_vec());
                        if is_english(pt.to_string()) {
                            // output.push(key.to_vec());
                            sender.send(key.to_vec());
                            break; // NOTE: I'm not sure if i wanter to break here
                        }
                    }
                    Err(_) => (),
                }
            }
            Err(_) => (),
        }
        // println!("{:?}", key);
        key = key.inc();
        if key.is_done() {
            break;
        }
    }
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
            Err(_) => break
        }
    }

    output
}
