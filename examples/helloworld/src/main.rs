mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reflection = poem_grpc::Reflection::new()
        .add_file_descriptor_set(poem_grpc::include_file_descriptor_set!(
            "file_descriptor_set.bin"
        ))
        .build();

    let server = gear_microkit::GrpcServer::new()
        .add_service(reflection)
        .add_service(services::hello::new());

    server.start().await?;
    Ok(())
}
