use std::cell::RefCell;
use std::io::{Cursor, Read, Write};
use data_rct::transmission::Stream;

pub struct MemoryStream {
    last_written_byte_length: usize,
    cursor: Cursor<Vec<u8>>
}

impl Stream for MemoryStream {}


impl MemoryStream {
    pub fn new() -> Self {
        return Self {
            last_written_byte_length: 0,
            cursor: Cursor::new(Vec::new())
        }
    }

    pub fn set_position(&mut self, position: u64) {
        self.cursor.set_position(position);
    }
}

impl Write for MemoryStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written_bytes = self.cursor.write(buf);

        if let Ok(written_bytes) = written_bytes {
            self.last_written_byte_length += written_bytes;
        }

        return written_bytes;
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return self.cursor.flush()
    }
}

impl Read for MemoryStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {

        return self.cursor.read(buf);
    }
}


#[test]
pub fn memory_stream() {
    let memory_stream = RefCell::new(MemoryStream::new());
    // let memory_stream = RefCell::clone(original_stream.borrow_mut());
    // let cloned = Rc::clone(&memory_stream);

    memory_stream.borrow_mut().write(&[4u8, 5u8, 6u8])
        .expect("Failed to write memory_stream");

    memory_stream.borrow_mut().set_position(0);

    let mut result = Vec::new();
    memory_stream.borrow_mut().read_to_end(&mut result)
        .expect("Failed to read memory_stream");

    assert_eq!(result.as_slice(), &[4u8, 5u8, 6u8]);

    // ====

    memory_stream.borrow_mut().write(&[2u8, 7u8, 9u8])
        .expect("Failed to write memory_stream");

    result = Vec::new();
    memory_stream.borrow_mut().read_to_end(&mut result)
        .expect("Failed to read memory_stream");

    assert_eq!(result.as_slice(), &[2u8, 7u8, 9u8]);

    // ====

    memory_stream.borrow_mut().write(&[2u8, 7u8, 9u8])
        .expect("Failed to write memory_stream");

    memory_stream.borrow_mut().write(&[1u8, 2u8, 0u8])
        .expect("Failed to write memory_stream");

    result = Vec::new();
    memory_stream.borrow_mut().read_to_end(&mut result)
        .expect("Failed to read memory_stream");

    assert_eq!(result.as_slice(), &[2u8, 7u8, 9u8, 1u8, 2u8, 0u8]);
}