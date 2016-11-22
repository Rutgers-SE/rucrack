use util::{u8_vector, read_file_from_arg, is_english, load_linux_dictionary};
use keypool::KeyPool;
use encryption::{encrypt, decrypt};

use std::sync::mpsc::{channel, Sender};
use std::env::args;
use std::thread;
use std::str::{from_utf8};



fn dc(cipher_text: &Vec<u8>, key: &KeyPool, sender: &Sender<Vec<u8>>, thread_count: i64) {
    // let mut output = vec![];
    let mut key = key.clone();
    loop {
        // println!("Thread ID: {}, key {:?}", thread_count, key.to_vec());
        match decrypt(&cipher_text[..], &key.to_vec()) {
            Ok(decrypted_data) => {
                match from_utf8(&decrypted_data) {
                    Ok(pt) => {
                        println!("thread {}: Found Key  {:?}", thread_count, key.to_vec());
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
pub fn crack(cipher_text: &Vec<u8>, dictionary: &Vec<String>, thread_count: i64) -> Vec<Vec<u8>> {
    let (tx, rx) = channel();

    // load the iv file
    let iv_bytes = read_file_from_arg(args().nth(1))
        .expect("Need to provide the IV file");

    // setup the key pool
    let iv = iv_bytes.len() as u8;
    let im = 16 - iv_bytes.len() as u8;
    let keys = KeyPool::generate_keys(thread_count, im); // TODO: change the `1` to a variable

    let mut threads = vec![]; // thread pool
    let mut thread_count = 0;

    for mut key in keys {
        key.static_ms_bytes = iv_bytes.clone();

        // Cloning to make it thread safe
        let tx = tx.clone();
        let cipher_text = cipher_text.clone();

        // println!("Starting with key {:?} and DMS Cap {}", key.to_vec(), key.dynamic_ms_cap);

        threads.push(thread::spawn(move || {
            dc(&cipher_text, &key, &tx, thread_count.clone());
        }));

        thread_count = thread_count + 1;
    }

    for t in threads {
        t.join();
    }
    tx.send(vec![1]);

    let mut output = vec![];

    'recieve: loop {
        // println!("Here");
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
