use util::{is_english};
use keypool::KeyPool;
use encryption::{decrypt};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::str::{from_utf8};

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
                            sender.send(key.to_vec()).unwrap();
                        }
                    }
                    Err(_) => {
                        invalid_count = invalid_count + 1;
                        ()
                    }

                }
            }
            Err(_) => {
                fail_count = fail_count + 1;
                ()
            }
        }
        key = key.inc();
        if key.is_done() {
            break;
        }
    }
    // // NOTE: For debugging
    // println!("Invalid string {} time(s)\nFailed decode {} times",
    //     invalid_count,
    //     fail_count);
}

// returns potential keys
pub fn crack(kp: KeyPool, cipher_text: Vec<u8>, thread_count: i64) -> Vec<Vec<u8>> {
    let keys = kp.split_key(thread_count);

    let mut threads = vec![]; // thread pool

    // com. channel
    let (tx, rx) = channel();
    for key in keys {

        // Cloning to make it thread safe
        let tx = tx.clone();
        let cipher_text = cipher_text.clone();

        threads.push(thread::spawn(move || {
            dc(&cipher_text, &key, &tx);
        }));
    }

    for t in threads {
        t.join().unwrap();
    }

    tx.send(vec![1]).unwrap(); // This is how to channel is notified to break the loop.

    let mut output = vec![]; // This will become the list of potential keys
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
