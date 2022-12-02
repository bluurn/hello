use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use serde::Deserialize;
use threadpool::ThreadPool;

fn default_port() -> u16 {
    1337
}

fn default_host() -> String {
    String::from("0.0.0.0")
}

fn default_pool_size() -> usize {
    4
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_host")]
    host: String,
    #[serde(default = "default_pool_size")]
    pool_size: usize,
}

impl Config {
    pub fn addr(&self) -> String {
        format!("{}:{}", &self.host, &self.port)
    }
}

fn main() {
    let config = envy::from_env::<Config>().unwrap();

    let listener = TcpListener::bind(&config.addr()).unwrap();
    let pool = ThreadPool::new(config.pool_size);

    println!(
        "Listening on {}, pool size: {}",
        config.addr(),
        config.pool_size
    );

    println!("Click http://{}/ to open it in the browser", config.addr());

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
    println!("Shutting down!");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match request_line.as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));

            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
