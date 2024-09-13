pub mod chat_common;

use crate::chat_common::{ChatMessage, MsgType, UsrMessage};
use std::env;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

fn chat_prompt() {
    print!(">>");
    let _ = std::io::stdout().flush();
}

fn get_std_input(mtx: Sender<String>) {
    let mut msg = String::new();
    loop {
        chat_prompt();
        let _ = std::io::stdin().read_line(&mut msg);
        if msg.trim() == "leave".to_string() {
            mtx.send(msg.clone()).unwrap();
            break;
        } else if msg.trim()[0..5].to_string() == "send ".to_string() {
            mtx.send((&msg.trim()[5..]).to_string()).unwrap();
        }
        msg = "".to_string();
    }
}

fn send_msg(mut cn: &TcpStream, mtype: MsgType, smsg: String) -> i32 {
    let m1 = ChatMessage {
        mtype: mtype,
        msg: smsg,
    };

    let data = bincode::serialize(&m1).unwrap();
    match cn.write(&data) {
        Err(e) => {
            println!("write failed, {}", e);
            -1
        }
        Ok(_m) => 0,
    };

    0
}

fn main() {
    let mut ubuff;
    let mut buf: Vec<u8> = vec![0; 100];
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <user id> <server_addr:port>", args[0]);
        return;
    }
    let srv_ep = &args[2];

    let (tx, rx) = mpsc::channel();
    let mut tcp_st: TcpStream;

    let _ = match TcpStream::connect(srv_ep) {
        Err(_e) => {
            println!("connect failed, {}", _e);
            return;
        }
        Ok(m) => {
            tcp_st = m;
        }
    };

    println!(
        "Starting Async Chat Client, connected to server on {}  ...",
        args[2]
    );
    println!("You can type:\n 1. 'send' then <text> for sending message to other ends\n 2. `leave` anytime to quit this chat");
    tcp_st
        .set_read_timeout(Some(Duration::from_millis(1)))
        .unwrap();
    let mut msg_type = MsgType::MsgChat;
    if send_msg(&tcp_st, MsgType::MsgJoin, args[1].clone()) == 0 {
        thread::spawn(|| get_std_input(tx));
        loop {
            let ok_rx = rx.try_recv();

            if !ok_rx.is_err() {
                ubuff = ok_rx.unwrap();
                if ubuff.trim() == "leave".to_string() {
                    msg_type = MsgType::MsgLeave;
                    ubuff = "".to_string();
                }
                if send_msg(&tcp_st, msg_type.clone(), ubuff) == -1 {
                    break;
                }
            }

            if msg_type == MsgType::MsgLeave {
                break;
            }

            let _ = match tcp_st.read(&mut buf) {
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        continue;
                    }
                    println!("read failed, {}", e.kind());
                    break;
                }
                Ok(m) => {
                    if m > 0 {
                        let m2: UsrMessage = bincode::deserialize(&buf).unwrap();
                        println!("");
                        println!("{}>{}", m2.from_user, m2.msg);
                        chat_prompt();
                    }
                }
            };
        }
    }

    println!("");
    println!("Disconnecting... !");
    tcp_st.shutdown(Shutdown::Both).expect("Shutdown failed");
}

#[cfg(test)]
mod tests {

    #[test]
    fn do_test1() {
        println!("testing ...");
    }
}
