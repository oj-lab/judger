use std::sync::{Arc, Mutex};

use tokio::time::{sleep, Duration};
use tonic::{transport::Server, Request, Response, Status};

use judger::greeter_server::{Greeter, GreeterServer};
use judger::judger_server::JudgerServer;
use judger::{HelloReply, HelloRequest};

pub mod judger {
    tonic::include_proto!("judger");
}

mod judge;

pub struct MyGreeter {
    ctx: Arc<Mutex<Context>>,
}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = judger::HelloReply {
            message: format!(
                "Hello {}!, heartbeat is {}",
                request.into_inner().name,
                self.ctx.lock().unwrap().heartbeat_count
            ),
        };
        Ok(Response::new(reply))
    }
}

struct Context {
    heartbeat_count: usize,
}

async fn heartbeat(ctx: Arc<Mutex<Context>>) {
    loop {
        sleep(Duration::from_secs(10)).await;
        ctx.lock().unwrap().heartbeat_count += 1;
        println!("Todo heartbeat {}", ctx.lock().unwrap().heartbeat_count);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Arc::new(Mutex::new(Context { heartbeat_count: 0 }));
    let addr = "[::]:50051".parse().unwrap();
    let greeter = MyGreeter { ctx: ctx.clone() };

    println!("GreeterServer listening on {}", addr);

    tokio::spawn(heartbeat(ctx.clone()));

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .add_service(JudgerServer::new(judge::MyJudger::default()))
        .serve(addr)
        .await?;

    Ok(())
}
