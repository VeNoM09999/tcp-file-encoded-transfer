use std::fs::File;
use std::io::{BufWriter, Write};
use std::rc::Rc;
use std::time::Instant;

use flate2::write::GzDecoder;

#[allow(dead_code)]
pub struct UploadSession {
    pub buffer: Vec<u8>,
    pub decoder: Option<GzDecoder<BufWriter<File>>>,
    pub file_path: Rc<str>,
    pub threshold: usize,
}
#[allow(dead_code, unused_variables)]
impl UploadSession {
    pub fn write(&mut self, compressed_chunk: &[u8]) {
        let start = Instant::now();
        self.buffer.extend_from_slice(compressed_chunk);

        //self.threshold += compressed_chunk.len();

        if self.buffer.len() >= self.threshold {
            self.write_to_disk();
        }
        let duration = start.elapsed();
        //println!(
        //   "Chunk decompressed and added in buffer : {:.3} ms ({} bytes)",
        // duration.as_secs_f64() * 1000.0,
        //compressed_chunk.len()
        //   )
    }
    pub fn write_to_disk(&mut self) {
        let start = Instant::now();
        if let Some(ref mut decoder) = self.decoder {
            let _ = decoder.write_all(&self.buffer);
            self.buffer.clear();
            let duration = start.elapsed();

            println!(
                "Buffer written to disk and cleared!: {:.3} ms)",
                duration.as_secs_f64() * 1000.0
            );
        }
    }
}
