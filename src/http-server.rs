mod http_request;
mod http_response;

use std::thread;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, Shutdown};
use crate::http_request::*;
use crate::http_response::*;
//use self::http_request;


const HOST: & str = "127.0.0.1:34254";


fn handle(request: &Request, response: &mut Response) -> Result<(), String>{
    response.version = request.version.clone();
    /*
    let res = request.bytes();
    //println!("{:?}", http_request::parse(request));
    //println!("{:?}", parse(request));
    return res;
    */
    Ok(())
}

fn handle_request(mut stream: TcpStream) {
    let mut buf = vec![0; 1024];

    let mut request = http_request::new();

    match stream.read(&mut buf) {
        Ok(n) => {
            if n == 0 {
                println!("shutdown");
                stream.shutdown(Shutdown::Both).unwrap();
                return;
            }

            println!("> {} {:?}", n, buf[0..n].to_vec());
            match request.parse(&mut buf[0..n].to_vec()) {
                Ok(()) => {},
                Err(e) => {
                    println!("{}", e); // TODO error response
                    stream.shutdown(Shutdown::Both).unwrap();
                    return;
                }
            }
        }
        Err(e) => {
            println!("ERR: {:?}", e); // TODO error response
            stream.shutdown(Shutdown::Both).unwrap();
            return;
        }
    }

    let response = &mut http_response::new();
    match handle(&request, response) {
        Ok(()) => {
            println!("{:?}", String::from_utf8(response.to_bytes()));
            stream.write(response.to_bytes().as_slice()).unwrap();
        }
        Err(e) => {
            println!("ERR: {:?}", e); // TODO error response
            stream.shutdown(Shutdown::Both).unwrap();
            return;
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(HOST)?;

    println!("starting ... {}", HOST);
    /*
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_request(&mut stream);
            }
            Err(e) => {
                println!("ERR: {:?}", e);
            }
        }
    }
    Ok(())
    */

    loop {
        let (stream, _) = listener.accept()?;
        thread::spawn(move || {
            handle_request(stream);
        });
    }
}
