use rust_book_server_example::{process_request, Database, LocalDatabase, ThreadPool};
use std::str;
use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
    sync::Arc,
};

fn main() {
    env_logger::init();

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    let db = Arc::new(LocalDatabase::new());

    db.set("test".to_string(), "1".to_string());

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let db = Arc::clone(&db);
        pool.execute(move || {
            handle_connection(stream, db);
        });
    }
}

fn handle_connection(mut stream: TcpStream, db: Arc<impl Database>) {
    // let mut buf_reader = BufReader::new(&mut stream);
    // Todo, workout when should use buf reader ? buffer
    // TODO - how does this work in rust and buffers TCP generally

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request = str::from_utf8(&buffer).unwrap();

    // Note, this is a hack and isn't complying with the HTTP spec
    // https://www.rfc-editor.org/rfc/rfc9112#name-message-body-length
    // Should be looking at the content-length instead
    let end = request.find('\0').unwrap();
    let request = request[..end].to_string();

    let response = process_request(request, db);

    stream.write_all(response.as_bytes()).unwrap();
}
