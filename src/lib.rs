use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};

use hard::{buffer_type, Buffer, BufferMut};

use rsa::pkcs8::der::zeroize::Zeroize;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};

use generic_array::GenericArray;

use serde_derive::{Deserialize, Serialize};

// re-export bincode
pub extern crate bincode;

#[derive(Serialize, Deserialize)]
pub struct AesPacket {
    nonce: [u8; 12],
    ciphertext: Vec<u8>,
}

impl AesPacket {
    pub fn encrypt(cipher: &mut Aes256Gcm, data: Vec<u8>) -> Self {
        let nonce = rand::random::<[u8; 12]>();
        let ciphertext = cipher.encrypt(&Nonce::from(nonce), &data[..]).unwrap();
        AesPacket {
            // NOTE: safe unwrap
            nonce: nonce.as_slice()[..12].try_into().unwrap(),
            ciphertext,
        }
    }
    pub fn decrypt(&self, cipher: &mut Aes256Gcm) -> Vec<u8> {
        cipher
            .decrypt(&Nonce::from(self.nonce), &self.ciphertext[..])
            .unwrap()
    }
    pub fn encrypt_to_bytes(cipher: &mut Aes256Gcm, data: Vec<u8>) -> Vec<u8> {
        bincode::serialize(&Self::encrypt(cipher, data)).unwrap()
    }
}

buffer_type!(
    HardPrivKey(1588);
);

// Arc<Mutex<HardPrivKey>> automatically makes everything Send + Sync, however
// it won't automatically become Send and so we have to mark it afaik
//
// src: https://users.rust-lang.org/t/solved-how-to-move-non-send-between-threads-or-an-alternative/19928/10
// src 2: i also ran like 20 clients at the same time doing KEX using `&` in zsh and it worked soo
// :skull: (also at high loads, when it didn't have over 20 threads active it performed the key
// exchanges sequentially, further proving that the mutex worked and therefore this is Send, but
// don't expose it as a public api, in this library code we're 100% sure its always behind a mutex
// but not in other people's code)
unsafe impl Send for HardPrivKey {}

type MutexHPK = Arc<Mutex<HardPrivKey>>;

pub struct StcpServer {
    rsa_private_key: MutexHPK,
    rsa_public_key: RsaPublicKey,
    pub listener: TcpListener,
}

impl StcpServer {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let mut rng = rand::thread_rng();

        let hard_private_key: MutexHPK = Arc::new(Mutex::new({
            let priv_key = RsaPrivateKey::new(&mut rng, 4096).expect("failed to gen?");
            let mut serialized_priv_key = bincode::serialize(&priv_key).unwrap();
            let mut hard_privkey = HardPrivKey::new().unwrap(); // size of serialized 4096 bit key by bincode
            hard_privkey.zero();
            hard_privkey.copy_from_slice(&serialized_priv_key[..]);

            serialized_priv_key.zeroize();

            hard_privkey
        }));

        let pubkey: RsaPublicKey = {
            let hpk = hard_private_key.lock().unwrap();
            bincode::deserialize::<RsaPrivateKey>(&(*hpk)[..])
                .unwrap()
                .to_public_key()
        };

        let listener = TcpListener::bind(addr)?;

        Ok(Self {
            rsa_private_key: hard_private_key,
            rsa_public_key: pubkey,
            listener,
        })
    }

    pub fn kex_with_stream(&self, stream: &mut TcpStream) -> io::Result<Aes256Gcm> {
        // send serialized public key
        let ser_pubkey = &bincode::serialize(&self.rsa_public_key).unwrap()[..];
        stream.write_all(ser_pubkey)?;

        // get encrypted aes key
        let mut enc_aes_key = [0_u8; 512];
        stream.read_exact(&mut enc_aes_key)?;

        // decrypt aes key
        let aes_key = {
            let hpk = self.rsa_private_key.lock().unwrap();
            let rsa_privkey = bincode::deserialize::<RsaPrivateKey>(&(*hpk)[..])
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            let padding = PaddingScheme::new_pkcs1v15_encrypt();
            let ser_aes_key = rsa_privkey
                .decrypt(padding, &enc_aes_key[..])
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            GenericArray::from_slice(&ser_aes_key[..]).to_owned()
        };

        Ok(Aes256Gcm::new(&aes_key))
    }
}

pub fn client_kex(stream: &mut TcpStream) -> io::Result<Aes256Gcm> {
    // get serialized public key
    let mut ser_pubkey = [0_u8; 532];
    stream.read_exact(&mut ser_pubkey)?;

    // deserialize pubkey
    let pubkey = bincode::deserialize::<RsaPublicKey>(&ser_pubkey[..]).unwrap();

    // generate aes key
    let aes_key = Aes256Gcm::generate_key(&mut OsRng);

    // encrypt aes key with pubkey
    let mut rng = rand::thread_rng();
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let encrypted_aes_key = pubkey
        .encrypt(&mut rng, padding, &(aes_key.to_vec())[..])
        .expect("encryption failed");

    // send encrypted aes key
    stream.write_all(&encrypted_aes_key[..])?;

    Ok(Aes256Gcm::new(&aes_key))
}
