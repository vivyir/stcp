use stcp::{client_kex, AesPacket};
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let mut file = File::open("./damn.mp3").unwrap();
    let mut filedata = Vec::with_capacity(file.metadata().unwrap().len() as usize);
    file.read_to_end(&mut filedata).unwrap();

    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 3333");
            let mut data = [0 as u8; 16384]; // 16KB buffer
            let mut filechunks = filedata.chunks(32_768 - 36);

            let mut aes_cipher = client_kex(&mut stream);
            // kex is now complete, you can communicate using `AesPacket`s

            // sending the first chunk
            let msg = AesPacket::encrypt_to_bytes(
                &mut aes_cipher,
                filechunks.next().unwrap().iter().copied().collect(),
            );
            stream.write(&msg).unwrap();

            while match stream.read(&mut data) {
                Ok(size) => {
                    if !&data.is_empty() {
                        /* this is where we would parse, deserialize and decrypt the packet just
                         * like in the server, but we don't care about the contents (they're \xff,
                         * just a single byte to signal that "hey, i got your packet"), and after
                         * finding out the packet was received we send the next one
                         *
                        let packet = bincode::deserialize::<AesPacket>(&data[0..size]).unwrap();
                        let decrypted_data = packet.decrypt(&mut aes_cipher);
                        */

                        match filechunks.next() {
                            Some(chunk) => {
                                let reply = AesPacket::encrypt_to_bytes(
                                    &mut aes_cipher,
                                    chunk.iter().copied().collect(),
                                );
                                stream.write(&reply).unwrap();
                                true
                            }
                            None => {
                                let reply =
                                    AesPacket::encrypt_to_bytes(&mut aes_cipher, b"close".to_vec());
                                stream.write(&reply).unwrap();
                                false
                            }
                        }
                    } else {
                        println!("Unexpected reply!");
                        false
                    }
                }
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                    false
                }
            } {}
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
