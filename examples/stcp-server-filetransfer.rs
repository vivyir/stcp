use aes_gcm::Aes256Gcm;
use stcp::{bincode, AesPacket, StcpServer};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream, mut aes_cipher: Aes256Gcm) {
    let mut data = [0 as u8; 32_768]; // using 32KB buffer
    let mut filebuf = vec![];

    while match stream.read(&mut data) {
        Ok(size) => {
            // parse (&data[0..size]), deserialize and decrypt AesPacket, this is stcp specific
            let packet = bincode::deserialize::<AesPacket>(&data[0..size]).unwrap();
            let decrypted_data = packet.decrypt(&mut aes_cipher);

            // try converting decrypted data into a string and printing it if successful, otherwise
            // skip
            let mut ddstr = String::from("");
            match std::str::from_utf8(&decrypted_data) {
                Ok(v) => {
                    println!("{}: {} bytes", v, &data[0..size].len());
                    ddstr = v.into();
                }
                Err(_) => {}
            }

            // rudimentary flushing system, not stcp specific, just usual networking stuff
            if ddstr == "close" {
                println!("total bytes received before closing: {}", filebuf.len());

                // return false to the while statement
                // otherwise this thread would NEVER die
                // (this took me an hour to figure out)
                false
            } else {
                // store the decrypted data in a buffer made beforehand
                filebuf.extend_from_slice(&decrypted_data[..]);

                // this is stcp specific, encrypt data with aes and return bytes
                let reply = AesPacket::encrypt_to_bytes(&mut aes_cipher, b"\xff".to_vec());

                stream.write(&reply).unwrap();

                // return true to keep the thread alive
                // and go to the next tcp read operation
                true
            }
        }
        Err(_) => {
            println!(
                "An error occurred, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn main() {
    // generates rsa4096 keypair as well as returning an instance of StcpServer
    let stcp_server = StcpServer::bind("0.0.0.0:3333").unwrap();
    println!("Server listening on port 3333");

    for stream in stcp_server.listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());

                let aes_cipher = stcp_server.kex_with_stream(&mut stream);
                println!(
                    "KEX complete with {} (got aes key)",
                    stream.peer_addr().unwrap()
                );

                thread::spawn(move || {
                    // connection succeeded
                    handle_client(stream, aes_cipher);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }

    // close the socket server
    drop(stcp_server);
}
