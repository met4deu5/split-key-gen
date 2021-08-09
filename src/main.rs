use sharks::{Share, Sharks};
use shkeleton::clap::{App, Arg, SubCommand, Values};
use rand::random;
use base64;
use z85;
use shkeleton::itertools::assert_equal;
use std::convert::TryFrom;

fn main() {
    let app = App::new("split-key-gen")
        .subcommand(SubCommand::with_name("gen-key").arg(Arg::with_name("random").help("32 random bytes in hex with spaces").required(true)))
        .subcommand(SubCommand::with_name("gen-key-part").arg(Arg::with_name("key-part").help("key parts").required(true).min_values(2).max_values(2)))
        .subcommand(SubCommand::with_name("recover").arg(Arg::with_name("key-part").help("key parts").required(true).min_values(2).max_values(2)));
    let sharks = Sharks(2);
    let matches = app.get_matches();
    match matches.subcommand() {
        ("gen-key", Some(args)) => {
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
            print_key(&external_random);
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
                let z85 = z85::encode(&data);
                let b64 = base64::encode(&data);
                assert_eq!(data, z85::decode(&z85).expect("can't decode z85"));
                assert_eq!(data, base64::decode(&b64).expect("can't decode base64"));
                println!("Key part {}: b64:{} | z85:{}", idx, b64, z85);
            }
        }
        ("recover", Some(args)) => {
            let key = recover_key(&sharks, args.values_of("key-part"));
            print_key(&key);
        }
        _ => panic!("unsupported")
    }
}

fn recover_key(sharks: &Sharks, values: Option<Values<'_>>) -> Vec<u8> {
    sharks.recover(values.expect("key-part arguments required").map(|s| {
        if let Some(s) = s.strip_prefix("z85:") {
            Share::try_from(z85::decode(s).expect("can't decode z85").as_slice()).expect("can't make share")
        } else if let Some(s) = s.strip_prefix("b64:") {
            Share::try_from(base64::decode(s).expect("can't decode base64").as_slice()).expect("can't make share")
        } else {
            panic!("can't decode key part: unknown format");
        }
    }).collect::<Vec<_>>().as_slice()).expect("can't recover")
}

fn print_key_part(key_part: &[u8]) {
    print_key_impl(key_part, "Key part");
}

fn print_key(key: &[u8]) {
    print_key_impl(key, "Key");
}

fn print_key_impl(key: &[u8], s: &str) {
    println!("{} data: {:x?}", s, key);
    println!("{}: b64:{} | z85:{}", s, base64::encode(key), z85::encode(key));
}

fn recover_key_for_key_parts() {

}
