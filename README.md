# learn-http

`learn-http` is a simple toy HTTP server. 

# Usage


1. Run a server.

```
% cargo run --bin http-server
```

2. Access with a simple client.

```
% cargo run --bin http-client
```

# Features

* Configuration
  * [x] Content root
  * [x] Host's IP address
  * [x] Listen port
  * [ ] Extension status code


* Header
  * [ ] Allow https://tools.ietf.org/html/rfc1945#section-10.1
  * [ ] Authorization https://tools.ietf.org/html/rfc1945#section-10.2
  * [ ] Content-Encoding https://tools.ietf.org/html/rfc1945#section-10.3
  * [x] Content-Length
  * [x] Content-Type
  * [x] Date
  * [ ] Expires https://tools.ietf.org/html/rfc1945#section-10.7
  * [x] From
  * [ ] If-Modified-Since
  * [x] Last-Modified
  * [ ] Pragma
  * [ ] Referer
  * [x] Server
  * [ ] User-Agent
  * [ ] WWW-authenticate
  
  
* Additinal Header
  * [ ] Accept
  * [ ] Accept-Charset
  * [ ] Accept-Encoding
  * [ ] Accept-Language
  * [ ] Link
  * [ ] MIME-Version
  * [ ] Retry-After
  * [ ] Title
  * [ ] URI
  
* Features
  * [ ] Access Authentication https://tools.ietf.org/html/rfc1945#section-11
