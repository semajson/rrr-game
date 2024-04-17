use rust_book_server_example::{process_request, Database, ThreadPool};
use std::{
    collections::HashMap,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let pool = ThreadPool::new(4);
    let db = Arc::new(Mutex::new(Database::new()));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let db = Arc::clone(&db);
        pool.execute(move || {
            handle_connection(stream, db);
        });
    }
}

fn handle_connection(mut stream: TcpStream, db: Arc<Mutex<Database>>) {
    let buf_reader = BufReader::new(&mut stream);

    let request = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let response = process_request(request, db);

    stream.write_all(response.as_bytes()).unwrap();
}
