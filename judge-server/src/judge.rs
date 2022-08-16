use std::{error::Error, io::ErrorKind, pin::Pin};

use tokio::sync::mpsc;
use tonic::{Streaming, Request, Response, Status};
use tokio_stream::{wrappers::ReceiverStream, StreamExt, Stream};

use crate::judger::{judger_server::Judger, JudgeRequest, JudgeReply};

fn match_for_io_error(err_status: &Status) -> Option<&std::io::Error> {
    let mut err: &(dyn Error + 'static) = err_status;

    loop {
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            return Some(io_err);
        }

        err = match err.source() {
            Some(err) => err,
            None => return None,
        };
    }
}

#[derive(Default)]
pub struct MyJudger {}

#[tonic::async_trait]
impl Judger for MyJudger {
    type JudgeStream = Pin<Box<dyn Stream<Item = Result<JudgeReply, Status>> + Send>>;

    async fn judge(
        &self,
        request: Request<Streaming<JudgeRequest>>,
    ) -> Result<Response<Self::JudgeStream>, Status> {
        println!("EchoServer::bidirectional_streaming_echo");

        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);

        // this spawn here is required if you want to handle connection error.
        // If we just map `in_stream` and write it back as `out_stream` the `out_stream`
        // will be drooped when connection error occurs and error will never be propagated
        // to mapped version of `in_stream`.
        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(_) => tx
                        .send(Ok(JudgeReply {time_used: 0, memory_used: 0, result: "unsupported".to_string(), error: "unsupported".to_string() }))
                        .await
                        .expect("working rx"),
                    Err(err) => {
                        if let Some(io_err) = match_for_io_error(&err) {
                            if io_err.kind() == ErrorKind::BrokenPipe {
                                // here you can handle special case when client
                                // disconnected in unexpected way
                                eprintln!("\tclient disconnected: broken pipe");
                                break;
                            }
                        }

                        match tx.send(Err(err)).await {
                            Ok(_) => (),
                            Err(_err) => break, // response was droped
                        }
                    }
                }
            }
            println!("\tstream ended");
        });

        // echo just write the same data that was received
        let out_stream = ReceiverStream::new(rx);

        Ok(Response::new(
            Box::pin(out_stream) as Self::JudgeStream,
        ))
    }
}