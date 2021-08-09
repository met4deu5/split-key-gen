use sharks::{Share, Sharks};
use shkeleton::clap::{App, Arg, SubCommand};
use rand::random;
use base64;
use shkeleton::itertools::assert_equal;
use std::convert::TryFrom;

const ALPHABET: &[u8] = b"QWERTYUIOP{}ASDFGHJKL:\"|ZCVBNM<>?qwertyuiop[]asdfghjkl;'\\zxcvbnm,./`1234567890-=~!@#$%^&*()_+<>";

fn main() {
    let app = App::new("split-key-gen")
        .subcommand(SubCommand::with_name("gen").arg(Arg::with_name("random").help("32 random bytes in hex with spaces").required(true)))
        .subcommand(SubCommand::with_name("recover").arg(Arg::with_name("key-part").help("key parts").required(true).min_values(2).max_values(2)));
    let matches = app.get_matches();
    match matches.subcommand() {
        ("gen", Some(args)) => {
            let matches = args;
            let internal_random = random::<[u8; 32]>();
            let mut external_random = matches.value_of("random").expect("random argument").split(' ').map(|s| u8::from_str_radix(s, 16).expect("can't parse hex")).collect::<Vec<_>>();
            if external_random.len() < 32 {
                panic!("not enough random provided, 32 bytes needed")
            }
            println!("Internal random: {:x?}", internal_random);
            external_random = external_random[0..32].into();
            println!("External random: {:x?}", external_random);
            for i in 0..32 {
                external_random[i] ^= internal_random[i];
            }
            println!("Key data: {:x?}", external_random);
            let key = bytes_to_password(&external_random);
            println!("Key: {}", key);
            let sharks = Sharks(2);
            let mut shares = sharks.dealer(&external_random).take(4).collect::<Vec<_>>();
            for (idx1, share1) in shares.iter().enumerate() {
                for (idx2, share2) in shares.iter().enumerate() {
                    if idx1 == idx2 {
                        continue;
                    }
                    let k = sharks.recover([share1, share2]).expect("can't recover");
                    assert_eq!(external_random, k);
                    print!("OK ");
                }
            }
            println!();
            for (idx, share) in shares.iter().enumerate() {
                let data = Vec::from(share);
                let b64 = base64::encode(&data);
                assert_eq!(data, base64::decode(&b64).expect("can't decode base64"));
                println!("Key part {}: b64:{}", idx, b64);
            }
        }
        ("recover", Some(args)) => {
            let sharks = Sharks(2);
            let key = sharks.recover(args.values_of("key-part").expect("key-part arguments required").map(|s| {
                let s = s.strip_prefix("b64:").unwrap_or(s);
                Share::try_from(base64::decode(s).expect("can't decode base64").as_slice()).expect("can't make share")
            }).collect::<Vec<_>>().as_slice()).expect("can't recover");
            println!("Key data: {:x?}", key);
            println!("Key: {}", bytes_to_password(&key));
        }
        _ => panic!("unsupported")
    }
}

fn bytes_to_password(bytes: &[u8]) -> String {
    bytes.iter().map(|b| char::from(ALPHABET[*b as usize % ALPHABET.len()])).collect()
}