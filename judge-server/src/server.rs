use tonic::{transport::Server, Request, Response, Status};

use judger::greeter_server::{Greeter, GreeterServer};
use judger::judger_server::JudgerServer;
use judger::{HelloReply, HelloRequest};

pub mod judger {
    tonic::include_proto!("judger");
}

mod judge;

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = judger::HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .add_service(JudgerServer::new(judge::MyJudger::default()))
        .serve(addr)
        .await?;

    Ok(())
}
