
//use std::io::Cursor;

//const G: u8 = 71;
//const E: u8 = 69;
//const T: u8 = 84;
const CR: u8 = 13;
const LF: u8 = 10;
const SP: u8 = 32;
const HT: u8 = 9;
const DQ: u8 = 34;

fn is_char(u: u8) -> bool {
    return u <= 127
}

fn is_ctl(u: u8) -> bool {
    return u == 127 || u <= 31
}

fn is_tspecial(u: u8) -> bool {
    let ts = vec![
        40, // "("
        41, // ")"
        60, // "<"
        62, // ">"
        64, // "@"
        44, // ","
        59, // ";"
        58, // ":"
        92, // "\\"
        34, // "\""
        47, // "/"
        91, // "["
        93, // "]"
        63, // "?"
        61, // "="
        123, // "{"
        125, // "}"
        SP,
        HT
    ];

    for i in ts {
        if i == u {
            return true;
        }
    }
    return false;
}

//fn tokenize(content: Vec<u8>)
//

#[derive(PartialEq, Clone, Debug)]
enum Version {
    V0_9,
    V1_0,
    V1_1,
}

#[derive(PartialEq, Clone, Debug)]
enum Method {
    NONE,
    GET,
    HEAD,
    POST,
}

#[derive(Clone, Debug)]
pub struct Request {
    //bytes: Cursor<Vec<u8>>,
    bytes: Vec<u8>,

    method: Method,
    uri: String,
    version: Version,
    header: Vec<HeaderEntry>,

    rest: String,

    idx: usize,
    space_count: u32,
    terminated: bool,
}

#[derive(Clone, Debug)]
pub struct HeaderEntry {
    field_name: String,
    field_value: String,
}

pub fn new() -> Request {
    Request {
        //bytes: Cursor::new(Vec::new()),
        bytes: Vec::new(),
        method: Method::NONE,
        uri: String::new(),
        version: Version::V0_9,
        header: Vec::new(),
        
        rest: String::new(),

        idx: 0,
        space_count: 0,
        terminated: false,
    }
}

impl Request {

    /*
    pub fn is_terminated(& self) -> bool {
        return self.terminated;
    }


    fn back(&mut self, l: usize) {
        self.idx -= l;
    }

    fn next(&mut self, l: usize) -> Option<&str> {
        if self.bytes.len() < self.idx + l {
            return None;
        }
        let s = Some(std::str::from_utf8(&self.bytes[self.idx..self.idx+l]).unwrap());
        self.idx += l;
        return s;
    }
    */

    fn skip_space(&mut self) {
        let mut length = 0;
        while self.idx + length < self.bytes.len() {
            match self.bytes[self.idx + length] {
                //SP | CR | LF => {
                SP => {
                    length+=1;
                }
                _ => {
                    break;
                }
            }
        }
        self.idx += length;
    }

    fn next_word(&mut self) -> Option<&str> {
        self.skip_space();
        let mut length = 1;
        if self.idx + length >= self.bytes.len() {
            return None;
        }
        while self.idx + length < self.bytes.len() {
            match self.bytes[self.idx + length] {
                SP => {
                    break;
                }
                CR => {
                    if length == 1 {
                        length += 1;
                    } else {
                        break;
                    }
                }
                LF => {
                    length += 1;
                    break;
                }
                _ => {
                    length +=1;
                }
            }
        }

        match std::str::from_utf8(&self.bytes[self.idx..self.idx+length]) {
            Ok(s) => {
                self.idx += length;
                Some(s)
            }
            Err(e) => {
                panic!(e);
            }
        }
    }


    fn next_token(&mut self) -> Option<&str> {
        let mut length = 0;
        while self.idx + length < self.bytes.len() {
            // token = 1*<any CHAR except CTLs or tspecials>
            let u = self.bytes[self.idx + length];
            if is_char(u) && !is_ctl(u) && !is_tspecial(u) {
                length += 1;
            } else {
                break;
            }
        }

        match std::str::from_utf8(&self.bytes[self.idx..self.idx+length]) {
            Ok(s) => {
                self.idx += length;
                Some(s)
            }
            Err(e) => {
                panic!(e);
            }
        }
    }

    fn try_lws(&self) -> Option<&str> {
        let u = self.bytes[self.idx];
        if u == SP || u == HT {
            let length = 1;
            return match std::str::from_utf8(&self.bytes[self.idx-length..self.idx]) {
                Ok(s) => Some(s),
                Err(_) => None,
            };
        }
        if self.idx + 2 < self.bytes.len() {
            let v = self.bytes[self.idx + 1];
            let w = self.bytes[self.idx + 2];
            if v == LF && (w == SP || w == HT) {
                let length = 3;
                return match std::str::from_utf8(&self.bytes[self.idx-length..self.idx]) {
                    Ok(s) => Some(s),
                    Err(_) => None,
                };
            }

            /*
            if v == LF && (w == SP || w == HT) {
                let mut length = 3;
                while  self.idx + length < self.bytes.len() {
                    let u = self.bytes[self.idx + length];
                    if u == SP || u == HT {
                        length += 1;
                    } else {
                        break;
                    }
                }

                return match std::str::from_utf8(&self.bytes[self.idx..self.idx+length]) {
                    Ok(s) => {
                        self.idx += length;
                        Some(s)
                    }
                    Err(_) => {
                        None
                    }
                };
            }
            */
        }
        return None;
    }

    fn parse_header_field_value(&mut self) -> Option<&str> {
        // field-value    = *( field-content | LWS )
        //
        // LWS            = [CRLF] 1*( SP | HT )
        //
        // field-content  = <the OCTETs making up the field-value
        //                  and consisting of either *TEXT or combinations
        //                  of token, tspecials, and quoted-string>
        //
        // TEXT           = <any OCTET except CTLs,
        //                  but including LWS>
        //
        // quoted-string  = ( <"> *(qdtext) <"> )
        //
        // qdtext         = <any CHAR except <"> and CTLs,
        //                  but including LWS>

        let is_quoted_string = self.bytes[self.idx] == DQ;


        if is_quoted_string {
            return Some("\"\"") // TODO
        } else {
            let mut length = 0;
            while self.idx + length < self.bytes.len() {
                let u = self.bytes[self.idx + length];
                if u == CR || u == SP || u == HT {
                    match self.try_lws() {
                        Some(s) => {
                            length += s.len();
                        }
                        None => {
                            break;
                        }
                    }
                } else if is_ctl(u) {
                    break;
                } else {
                    length += 1;
                }
            }

            if length > 0 {
                self.idx += length;
                return match std::str::from_utf8(&self.bytes[self.idx-length..self.idx]) {
                    Ok(s) => Some(s),
                    Err(_) => None,
                };
            }

        }


        None
    }

    fn parse_header_entry(&mut self) -> Result<(), String> {
        // HTTP-header = field-name ":" [ field-value ] CRLF

        let mut header_entry = HeaderEntry{
            field_name: String::new(),
            field_value: String::new(),
        };
        match self.next_token() {
            Some(s) => {
                header_entry.field_name = s.to_string();
            }
            None => {
                return Err("Error: filed name of request header.".to_string());
            }
        };

        self.idx += 1; // Expect ':'.

        match self.parse_header_field_value() {
            Some(s) => {
                header_entry.field_value = s.to_string();
            }
            None => {
                return Err("Error: filed value of request header.".to_string());
            }
        };

        self.header.push(header_entry);

        Ok(())
    }

    pub fn parse(&mut self, content: &mut Vec<u8>) -> Result<(), String> {
        if self.bytes.len() > 0 {
            return Ok(()); // already done.
        }

        self.bytes.append(content);

        if self.bytes.len() < 4 {
            return Err("The content is too short.".to_string());
        }

        match self.next_word() {
            Some("GET") => {
                self.method = Method::GET;
            }
            Some("HEAD") => {
                self.method = Method::HEAD;
            }
            Some("POST") => {
                self.method = Method::POST;
            }
            m => {
                return Err(format!("The content has an unknown method: {:?}", m));
            }
        }

        match self.next_word() {
            Some(s) => {
                self.uri = s.to_string();
            }
            None => {
                return Err("illegal state".to_string());
            }
        }

        match self.next_word() {
                Some("HTTP/1.0") => {
                    self.version = Version::V1_0;
                }
                Some("HTTP/1.1") => {
                    self.version = Version::V1_1;
                }
                Some("\r\n") => {
                    self.version = Version::V0_9;
                    self.terminated = true;
                    return Ok(());
                }
                a => {
                    return Err(format!("invalid token: {:?}", a));
                }
        }

        self.idx += 2; // CR LF

        if self.idx < self.bytes.len() {
            for b in self.bytes[self.idx..].iter() {
                self.rest.push(char::from(*b));
            }
        }

        while let Ok(()) = self.parse_header_entry() {
        }

        // FIX
        self.terminated = true;

        println!("debug: {:?}", self);
        return Ok(());
    }
    
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}
