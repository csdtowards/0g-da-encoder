mod service;

pub use service::{encoder::EncodeBlobReply, EncoderServer, EncoderService};

use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::info;

const MESSAGE_SIZE_LIMIT: usize = 1024 * 1024 * 1024; // 1G

pub async fn run_server(
    addr: SocketAddr, param_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let encoder_service = EncoderService::new(param_dir);
    info!("Encoder service ready");
    Server::builder()
        .add_service(
            EncoderServer::new(encoder_service)
                .max_decoding_message_size(MESSAGE_SIZE_LIMIT)
                .max_encoding_message_size(MESSAGE_SIZE_LIMIT),
        )
        .serve(addr)
        .await?;
    Ok(())
}
