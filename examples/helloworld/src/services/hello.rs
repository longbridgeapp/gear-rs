use gear_microkit::RequestExt;
use poem_grpc::{Request, Response, Status};

poem_grpc::include_proto!("hello");

#[derive(Debug, Clone)]
pub(crate) struct HelloWorldService;

impl Helloworld for HelloWorldService {
    async fn say_hello(&self, req: Request<HelloRequest>) -> Result<Response<HelloReply>, Status> {
        let message = match req.member_id() {
            Some(member_id) => format!("Hello {}({})!", req.name, member_id),
            None => format!("Hello {}!", req.name),
        };
        Ok(Response::new(HelloReply { message }))
    }
}

pub(crate) fn new() -> HelloworldServer<HelloWorldService> {
    HelloworldServer::new(HelloWorldService)
}
