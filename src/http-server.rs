extern crate toml;

mod conf;
mod http_request;
mod http_response;
mod method;
mod status;
mod util;

use std::fs;
use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::Path;
use std::thread;

use chrono::Local;
use serde::Deserialize;
//use serde_derive::Deserialize;

use crate::http_request::*;
use crate::http_response::*;
//use crate::conf;

// TODO cli option
const CONF_PATH: &str = "server_conf.json";

const ACCESS_CONF: &str = ".access";


#[derive(Debug, Deserialize)]
struct AccessConfig {
    auth: Option<AuthConfig>,
}

#[derive(Debug, Deserialize)]
struct AuthConfig {
    auth_type: String,
    auth_name: String,
    pass_file: String,
}

fn handle_content_info(response: &mut Response, access_path: & String) {
    match util::extension(&access_path) {
        Some("ico") => {
            response.add_header("Content-Type", "image/x-icon".to_string());
            response.add_header("Content-Length", format!("{}", response.entity_body.len()));
        }
        _ => {
            response.add_header("Content-Type", "text/html".to_string());
        }
    }
}

fn handle(request: &Request, response: &mut Response) -> Result<(), String> {
    response.version = request.version.clone();
    response.set_host(format!("{}:{}", conf::ip(), conf::port()));
    response.set_server(conf::server());
    let date_str = Local::now().to_rfc2822();
    response.add_header("Date", format!("{} GMT", &date_str[..date_str.len() - 6]));


    match request.uri.as_str() {
        "/debug" => {
            response.entity_body.append(&mut request.bytes());
        }
        request_uri => {
            // check for directory traversal attack
            let uri = match util::canonicalize(request_uri) {
                Some(s) => s,
                None => {
                    println!(
                        "debug(403)1: {}",
                        format!("{}{}", conf::root(), request_uri)
                    );
                    response.status = status::FORBIDDEN;
                    return Ok(());
                }
            };


            let access_path = format!("{}{}", conf::root(), uri);
            let path = Path::new(&access_path);

            // read a configuration file.
            let config_path = if path.is_dir() {
                path.with_file_name(ACCESS_CONF)
            } else {
                path.parent().unwrap().with_file_name(ACCESS_CONF)
            };
            let mut config_content = String::new();
            util::read_file(&config_path.to_str().unwrap().to_string(), &mut config_content).unwrap();
            //let access_config = toml::from_str(config_content.as_str());



            // TODO access a directory.
            // FIXME remove unnecessary clone.
            match fs::File::open(access_path.clone()) {
                Ok(mut file) => {
                    let mut buffer = Vec::new();
                    match file.read_to_end(&mut buffer) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("debug(403)2: {} {}", format!("{}{}", conf::root(), uri), e);
                            response.status = status::FORBIDDEN;
                            return Ok(());
                        }
                    }
                    response.entity_body.append(&mut buffer);
                }
                Err(_) => {
                    println!("debug(404): {}", format!("{}{}", conf::root(), uri));
                    response.status = status::NOT_FOUND;
                    return Ok(());
                }
            }

            // Last-Modified
            // TODO
            //   An origin server must not send a Last-Modified date which is later
            //   than the server's time of message origination. In such cases, where
            //   the resource's last modification would indicate some time in the
            //   future, the server must replace that date with the message
            //   origination date.
            match util::modified(&access_path) {
                Ok(t) => {
                    response.modified_datetime = Some(t);
                    response.add_header("Last-Modified", util::datetime_to_http_date(&t));
                }
                Err(_) => {
                    println!("debug(503)1: {}", format!("{}{}", conf::root(), uri));
                    response.status = status::INTERNAL_SERVER_ERROR;
                    return Ok(());
                }
            };

            match request.if_modified_since() {
                Some(s) => {
                    match response.modified_datetime {
                        Some(t) => {
                            if s > t {
                                response.status = status::NOT_MODIFIED;
                                return Ok(());
                            }
                        }
                        None => {
                            // already returned at the checkpoint for Last-Modified.
                        }
                    }
                }
                None => {
                    // do nothing
                }
            };

            handle_content_info(response, &access_path);
        }
    }

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

            match request.parse(&mut buf[0..n].to_vec()) {
                Ok(()) => {}
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
            //println!("response: {:?}", String::from_utf8(response.to_bytes()));
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
    // read a configuration file.
    let server_conf = conf::load(CONF_PATH);
    conf::set(server_conf.clone());

    let listener = TcpListener::bind(format!("127.0.0.1:{}", conf::port()))?;

    match listener.local_addr() {
        Ok(addr) => {
            println!("starting ... {}:{}", addr.ip(), addr.port());
        }
        Err(e) => {
            panic!("Error(local_addr): {}", e);
        }
    }

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

        let proc = |cnf, stream| {
            return || {
                conf::set(cnf);
                handle_request(stream);
            };
        };
        thread::spawn(proc(server_conf.clone(), stream));
    }
}
