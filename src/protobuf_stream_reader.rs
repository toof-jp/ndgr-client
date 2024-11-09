use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Default)]
pub struct ProtobufStreamReader {
    pub buffer: BytesMut,
}

impl ProtobufStreamReader {
    pub fn push_chunk(&mut self, chunk: &[u8]) {
        self.buffer.put(chunk);
    }

    fn read_length(&self) -> Option<(usize, usize)> {
        let mut offset = 0;
        let mut length = 0;
        let mut i = 0;

        loop {
            if offset >= self.buffer.len() {
                return None;
            }

            let current = self.buffer[offset];
            length |= ((current & 0x7F) as usize) << i;
            offset += 1;
            i += 7;

            if (current & 0x80) == 0 {
                break;
            }
        }

        Some((offset, length))
    }

    pub fn get_message(&mut self) -> Option<Bytes> {
        let (offset, length) = self.read_length()?;

        if offset + length > self.buffer.len() {
            return None;
        }

        let mut bytes = self.buffer.split_to(offset + length);
        bytes.advance(offset);
        Some(bytes.freeze())
    }
}
