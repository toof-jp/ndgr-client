use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &["nicolive-comment-protobuf/proto/dwango/nicolive/chat/service/edge/payload.proto"],
        &["nicolive-comment-protobuf/proto/"],
    )?;
    Ok(())
}
