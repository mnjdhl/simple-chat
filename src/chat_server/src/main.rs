pub mod chat_common;

use crate::chat_common::{ChatMessage, MsgType, UsrMessage};
use bincode;
use std::env;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

struct ConnCache {
    user_id: String,
    conn: TcpStream,
}

fn handle_connections(crx: Receiver<TcpStream>) {
    let mut conns: Vec<ConnCache> = Vec::new();

    println!("Starting connection handler...");
    loop {
        let ok_rx = crx.try_recv();

        if !ok_rx.is_err() {
            let cn = ok_rx.unwrap();
            cn.set_read_timeout(Some(Duration::from_millis(1))).unwrap();
            let cc = ConnCache {
                user_id: "".to_string(),
                conn: cn,
            };
            conns.push(cc);
            println!("Added a new client connection");
        }

        for i in 0..conns.len() {
            let mut data: Vec<u8> = vec![0; 100];
            let _ = match conns[i].conn.read(&mut data) {
                Err(_e) => {
                    continue;
                }
                Ok(m) => {
                    if m > 0 {
                        println!("Read {} bytes of data from user {}", m, conns[i].user_id);
                        let m1: ChatMessage = bincode::deserialize(&data).unwrap();
                        match m1.mtype {
                            MsgType::MsgJoin => {
                                /* Make sure that user is unique */
                                let mut flag = false;
                                for j in 0..conns.len() {
                                    if i != j && conns[j].user_id == m1.msg {
                                        flag = true;
                                        break;
                                    }
                                }
                                if flag {
                                    println!("Disconnecting user {}", m1.msg);
                                    conns[i]
                                        .conn
                                        .shutdown(Shutdown::Both)
                                        .expect("Shutdown failed");
                                    conns.remove(i);
                                    continue;
                                }
                                conns[i].user_id = m1.msg.clone();
                            }
                            MsgType::MsgChat => {
                                println!("Sending the new message to all others");
                                let dmsg = UsrMessage {
                                    from_user: conns[i].user_id.clone(),
                                    msg: m1.msg.clone(),
                                };
                                let data = bincode::serialize(&dmsg).unwrap();
                                for k in 0..conns.len() {
                                    if k != i {
                                        match conns[k].conn.write(&data) {
                                            Err(_e2) => {
                                                continue;
                                            }
                                            Ok(_m2) => {
                                                continue;
                                            }
                                        };
                                    }
                                }
                            }
                            MsgType::MsgLeave => {
                                println!("User {} leaving", conns[i].user_id);
                                conns[i]
                                    .conn
                                    .shutdown(Shutdown::Both)
                                    .expect("Shutdown failed");
                                conns.remove(i);
                                /* Go back to main loop */
                                break;
                            }
                        }
                    }
                }
            };
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <port>", args[0]);
        return;
    }
    let (tx, rx) = mpsc::channel();
    let tcp_listener: TcpListener;

    let _ = match TcpListener::bind("0.0.0.0:".to_string() + args[1].as_str()) {
        Err(_e) => {
            println!("bind failed, {}", _e);
            return;
        }
        Ok(m) => {
            println!("Starting Async Chat Server at TCP port {}...", args[1]);
            tcp_listener = m;
        }
    };

    /* Spawning handler thread */
    thread::spawn(|| handle_connections(rx));

    /*Main thread */
    for strm in tcp_listener.incoming() {
        let st = strm.unwrap();
        println!("Connection established and now sending it to handler!");
        tx.send(st).unwrap();
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn do_test1() {
        println!("testing ...");
    }
}
