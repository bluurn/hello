use std::{
    env, fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use threadpool::ThreadPool;

fn fetch_env(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(key) => key,
        Err(_) => String::from(default),
    }
}

struct Config {
    address: String,
    pool_size: usize,
}

impl Config {
    fn new() -> Config {
        let host = fetch_env("HOST", "0.0.0.0");
        let port = fetch_env("PORT", "1337").parse().unwrap_or(1337);
        let address = format!("{}:{}", host, port);
        let pool_size: usize = fetch_env("POOL_SIZE", "4").parse().unwrap_or(4);
        Config { address, pool_size }
    }
}

fn main() {
    let config = Config::new();
    let listener = TcpListener::bind(&config.address).unwrap();
    let pool = ThreadPool::new(config.pool_size);

    println!(
        "http://{}/ started ({} threads)",
        &config.address, &config.pool_size
    );

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
