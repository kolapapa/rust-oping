use std::env::args;

extern crate oping;
use oping::Ping;

macro_rules! perror {
    ($e: expr, $prefix: expr) => (
        match $e {
            Ok(x) => x,
            Err(e) => {
                println!("{}: {:?}", $prefix, e);
                std::process::exit(1);
            }
        }
    )
}

fn main() {
    if args().len() == 1 {
        println!("Usage: rustping hostname1 hostname2 ...");
        std::process::exit(0);
    }

    let mut p = Ping::new();
    perror!(p.set_timeout(5.0), "setting timeout");

    for host in args().skip(1) {
        println!("Pinging: {}", host);
        perror!(p.add_host(&host), "creating ping socket for host");
    }

    perror!(p.send(), "sending ping");

    let mut retcode = 0;
    for resp in p.iter() {
        println!("Response: {:?}", resp);
        if resp.dropped > 0 {
            retcode = 1;
        }
    }

    std::process::exit(retcode);
}
