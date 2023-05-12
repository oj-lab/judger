use actix_web::{get, web, App, HttpServer, Responder};

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    tokio::spawn(async move {
        // Suppose to send heartbeat here to a remote host
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            println!("Hello from Tokio!");
        }
    });

    HttpServer::new(|| App::new().service(greet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
