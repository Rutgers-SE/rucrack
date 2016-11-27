use util::is_english;
use keypool::KeyPool;
use encryption::decrypt;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::str::from_utf8;
// NOTE: mid kill attempt
// use celix::pthread_cancel;
// use std::os::unix::thread::JoinHandleExt;

fn dc(cipher_text: &Vec<u8>,
      key: &KeyPool,
      sender: &Sender<Vec<u8>>,
      tdone: &Sender<bool>)
      -> KeyPool {
    let mut key = key.clone();
    match decrypt(&cipher_text[..], &key.to_vec()) {
        Ok(decrypted_data) => {
            match from_utf8(&decrypted_data) {
                Ok(pt) => {
                    println!("Msg found \"{}\"", pt);
                    if is_english(pt.to_string()) {
                        match sender.send(key.to_vec()) {
                            Ok(_) => tdone.send(true).unwrap(),
                            Err(e) => panic!("Sorry master.. I fucked up... -- {}", e),
                        }
                    }

                }
                Err(_) => {
                    tdone.send(false).unwrap();
                }

            }
        }
        Err(_) => {
            tdone.send(false).unwrap();
        }
    }
    key.inc()
    // // NOTE: For debugging
    // println!("Invalid string {} time(s)\nFailed decode {} times",
    //     invalid_count,
    //     fail_count);
}

// returns potential keys
pub fn crack(kp: KeyPool, cipher_text: Vec<u8>, thread_count: i64) -> Vec<Vec<u8>> {
    let keys = kp.split_key(thread_count);

    let mut threads = vec![]; // thread pool

    // com. channels
    let (tx, rx) = channel();
    let (tdone, rdone) = channel::<bool>();
    let (tkill, rkill) = channel();

    for key in keys {
        let mut key = key.clone();
        // Cloning to make it thread safe
        let tx = tx.clone();
        let tdone = tdone.clone();
        let cipher_text = cipher_text.clone();

        threads.push(thread::spawn(move || {
            loop {
                // match rkill.try_recv() {
                //     Ok(_) | Err(TryRecvError::Disconnected) => {
                //         println!("Thread killed");
                //         break;
                //     }
                //     Err(TryRecvError::Empty) => {
                key = dc(&cipher_text, &key, &tx, &tdone);
                if key.is_done() {
                    break;
                }
                //     }
                // }
            }
        }));
    }

    let mut output = vec![]; // This will become the list of potential keys

    loop {
        match rdone.recv() {
            Ok(value) if value => {
                // Yeah. I want to panic if no vector (not a part of the plan)
                let v = rx.recv().unwrap();
                tkill.send(()).unwrap();
                output.push(v);
                println!("Done!");
                // unsafe {
                //     for t in threads {
                //         let pt = t.into_pthread_t();
                //         pthread_cancel(pt);
                //     }
                // }
                break;
            }
            _ => (),
        }
    }

    for t in threads {
        t.join().unwrap();
    }

    // tx.send(vec![1]).unwrap(); // This is how to channel is notified to break the loop.

    // loop {
    //     match rx.recv() {
    //         Ok(vector) => {
    //             if vector.len() == 1 {
    //                 break;
    //             }
    //             output.push(vector);
    //         }
    //         Err(_) => break,
    //     }
    // }

    output
}
