use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/communication.proto"], &["src/"])?;
    prost_build::compile_protos(&["src/discovery.proto"], &["src/"])?;

    return Ok(());
}
