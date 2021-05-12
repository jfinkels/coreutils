use std::io::BufRead;

pub fn zlines<B>(buf: B) -> ZLines<B> {
    ZLines { buf }
}

pub struct ZLines<B> {
    buf: B,
}

const ZERO: u8 = 0;

impl<B: BufRead> Iterator for ZLines<B> {
    type Item = std::io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<std::io::Result<Vec<u8>>> {
        let mut buf = Vec::new();
        match self.buf.read_until(ZERO, &mut buf) {
            Ok(0) => None,
            Ok(_) => Some(Ok(buf)),
            Err(e) => Some(Err(e)),
        }
    }
}
