use tiny_http::{Response, Server};

// might need to be async
pub fn start_server(port: u16) {
    let server = Server::http(format!("127.0.0.1:{}", port)).unwrap();

    for request in server.incoming_requests() {
        println!(
            "received request! method: {:?}, url: {:?}, headers: {:?}",
            request.method(),
            request.url(),
            request.headers()
        );

        let response = Response::from_string("hello world");
        request.respond(response);
    }
}
