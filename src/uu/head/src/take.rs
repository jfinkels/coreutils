//! Take all but the last elements of an iterator or sequential reader.
use std::io::Read;
use uucore::ringbuffer::RingBuffer;

/// Create an iterator over all but the last `n` elements of `iter`.
///
/// # Examples
///
/// ```rust,ignore
/// let data = [1, 2, 3, 4, 5];
/// let n = 2;
/// let mut iter = take_all_but(data.iter(), n);
/// assert_eq!(Some(4), iter.next());
/// assert_eq!(Some(5), iter.next());
/// assert_eq!(None, iter.next());
/// ```
pub fn take_all_but<I: Iterator>(iter: I, n: usize) -> TakeAllBut<I> {
    TakeAllBut::new(iter, n)
}

/// An iterator that only iterates over the last elements of another iterator.
pub struct TakeAllBut<I: Iterator> {
    iter: I,
    buf: RingBuffer<<I as Iterator>::Item>,
}

impl<I: Iterator> TakeAllBut<I> {
    pub fn new(mut iter: I, n: usize) -> TakeAllBut<I> {
        // Create a new ring buffer and fill it up.
        //
        // If there are fewer than `n` elements in `iter`, then we
        // exhaust the iterator so that whenever `TakeAllBut::next()` is
        // called, it will return `None`, as expected.
        let mut buf = RingBuffer::new(n);
        for _ in 0..n {
            let value = match iter.next() {
                None => {
                    break;
                }
                Some(x) => x,
            };
            buf.push_back(value);
        }
        TakeAllBut { iter, buf }
    }
}

impl<I: Iterator> Iterator for TakeAllBut<I>
where
    I: Iterator,
{
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<<I as Iterator>::Item> {
        match self.iter.next() {
            Some(value) => self.buf.push_back(value),
            None => None,
        }
    }
}

/// Return an adaptor that reads all but the last `n` bytes from a reader.
///
/// This function returns a new instance of [`Read`] that reads all but
/// the last `n` bytes, after which it will always return [`Ok`](0),
/// representing the end of the file (EOF).
///
/// # Examples
///
/// ```rust,ignore
/// use std::io::Cursor;
///
/// let mut reader = read_all_but(Cursor::new(b"vwxyz"), 2);
/// let mut buf = vec![];
/// reader.read_to_end(&mut buf).unwrap();
/// assert_eq!(buf, b"vwx");
/// ```
pub fn read_all_but<R: Read>(reader: R, n: usize) -> ReadAllBut<R> {
    ReadAllBut::new(reader, n)
}

/// A reader adaptor that reads all but the last bytes from a given reader.
pub struct ReadAllBut<R> {
    reader: R,
    ring_buffer: RingBuffer<u8>,
}

impl<R: Read> ReadAllBut<R> {
    pub fn new(mut reader: R, n: usize) -> ReadAllBut<R> {
        // Create a new ring buffer and fill it up.
        //
        // If there are fewer than `n` bytes in `reader`, then we
        // exhaust the reader so that whenever ReadAllBut::next()` is
        // called, it will return `None`, as expected.
        let mut buf = vec![0u8; n];
        let ring_buffer = match reader.read(&mut buf) {
            Ok(m) => RingBuffer::from_iter(buf[0..m].iter().copied(), n),
            Err(_) => RingBuffer::new(n),
        };
        ReadAllBut {
            reader,
            ring_buffer,
        }
    }
}

impl<R: Read> Read for ReadAllBut<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut tmp = vec![0u8; buf.len()];
        match self.reader.read(&mut tmp) {
            Ok(m) => {
                let mut i = 0;
                for b in tmp[0..m].iter() {
                    if let Some(out_byte) = self.ring_buffer.push_back(*b) {
                        buf[i] = out_byte;
                        i += 1;
                    }
                }
                Ok(i)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {

    mod take_all_but {

        use crate::take::take_all_but;

        #[test]
        fn test_fewer_elements() {
            let mut iter = take_all_but([0, 1, 2].iter(), 2);
            assert_eq!(Some(&0), iter.next());
            assert_eq!(None, iter.next());
        }

        #[test]
        fn test_same_number_of_elements() {
            let mut iter = take_all_but([0, 1].iter(), 2);
            assert_eq!(None, iter.next());
        }

        #[test]
        fn test_more_elements() {
            let mut iter = take_all_but([0].iter(), 2);
            assert_eq!(None, iter.next());
        }

        #[test]
        fn test_zero_elements() {
            let mut iter = take_all_but([0, 1, 2].iter(), 0);
            assert_eq!(Some(&0), iter.next());
            assert_eq!(Some(&1), iter.next());
            assert_eq!(Some(&2), iter.next());
            assert_eq!(None, iter.next());
        }
    }

    mod read_all_but {

        use crate::take::read_all_but;
        use std::io::{Cursor, Read};

        #[test]
        fn test_fewer_bytes() {
            let mut reader = read_all_but(Cursor::new(b"xyz"), 2);
            let mut buf = vec![];
            reader.read_to_end(&mut buf).unwrap();
            assert_eq!(buf, b"x");
        }

        #[test]
        fn test_same_number_of_bytes() {
            let mut reader = read_all_but(Cursor::new(b"xy"), 2);
            let mut buf = vec![];
            reader.read_to_end(&mut buf).unwrap();
            assert_eq!(buf.is_empty(), true);
        }

        #[test]
        fn test_more_bytes() {
            let mut reader = read_all_but(Cursor::new(b"x"), 2);
            let mut buf = vec![];
            reader.read_to_end(&mut buf).unwrap();
            assert_eq!(buf.is_empty(), true);
        }

        #[test]
        fn test_zero_bytes() {
            let mut reader = read_all_but(Cursor::new(b"xyz"), 0);
            let mut buf = vec![];
            reader.read_to_end(&mut buf).unwrap();
            assert_eq!(buf, b"xyz");
        }
    }
}
