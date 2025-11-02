use std::io::Write;

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum CompressionMethod {
    GZIP,
    BROTLI,
    NONE,
}

pub fn compress(method: CompressionMethod, data: &[u8]) -> Vec<u8> {
    match method {
        CompressionMethod::BROTLI => {
            // Brotli
            let mut encoder = brotli::CompressorWriter::new(Vec::new(), 4096, 5, 22);
            encoder.write_all(data).unwrap();
            let data = encoder.into_inner();
            data
        }
        CompressionMethod::GZIP => {
            // Gzip
            use flate2::Compression;
            use flate2::write::GzEncoder;
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(data).unwrap();
            encoder.finish().unwrap()
        }
        CompressionMethod::NONE => {
            // No compression
            data.to_vec()
        }
    }
}

pub fn get_header_name(method: CompressionMethod) -> &'static str {
    match method {
        CompressionMethod::BROTLI => "br",
        CompressionMethod::GZIP => "gzip",
        CompressionMethod::NONE => "",
    }
}
