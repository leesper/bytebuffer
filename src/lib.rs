use std::vec::Vec;
use std::mem;
use std::ptr;
use std::io::{Read, Result};

/// A byte buffer class modeled after muduo::net::Buffer
///
/// +-------------------+------------------+------------------+
/// | prependable bytes |  readable bytes  |  writable bytes  |
/// |                   |     (CONTENT)    |                  |
/// +-------------------+------------------+------------------+
/// |                   |                  |                  |
/// 0      <=      readerIndex   <=   writerIndex    <=     size
///

#[derive(Debug)]
pub struct Buffer {
	read_index: 	usize,
	write_index:	usize,
	data:			Vec<u8>,
}

/// 8 bytes prependable by default
pub const PREPEND: usize = 8;
/// Contains 1024 bytes by default
pub const INITIAL: usize = 1024;

impl Buffer {
	/// Returns a byte buffer of initial size if given
	///
	/// # Examples
	/// 
	/// ```
	/// use bytebuffer::Buffer;
	///
	/// let mut buf = Buffer::new(None);
	/// ```
	pub fn new(initial: Option<usize>) -> Buffer { 
		 let size = match initial {
		 	Some(sz) => sz,
		 	None => INITIAL,
		 };
		 Buffer {
		 	read_index: PREPEND,
		 	write_index: PREPEND,
		 	data: vec![0u8; PREPEND + size],
		 }
	}
	
	/// Swaps between two buffers
	pub fn swap(&mut self, other: &mut Self) { mem::swap(self, other); }
	
	/// Adjusts the size of buffer
	pub fn shrink(&mut self, reserve: usize) {
		let mut other: Buffer = Buffer::new(None);
		other.ensure_writable_bytes(self.readable_bytes() + reserve);
		other.append_string(&self.retrieve_all_as_string());
		self.swap(&mut other);
	}
	
	/// Returns how many bytes can read
	pub fn readable_bytes(&self) -> usize { self.write_index - self.read_index }
	
	/// Returns how many bytes can write
	pub fn writable_bytes(&self) -> usize { self.data.len() - self.write_index }
	
	/// Returns how many bytes can prepend
	pub fn prependable_bytes(&self) -> usize { self.read_index }
	
	/// Finds "\r\n" in buffer
	pub fn find_crlf(&self) -> Option<usize> {
		self.do_find_crlf(self.read_index, self.write_index)
	}
	
	pub fn find_crlf_from(&self, start: usize) -> Option<usize> {
		assert!(self.read_index <= start);
		assert!(start <= self.write_index);
		self.do_find_crlf(start, self.write_index)
	}
	
	fn do_find_crlf(&self, start: usize, end: usize) -> Option<usize> {
		assert!(start <= end);
		for i in start..end-1 {
			let chr1 = self.data[i] as char;
			let chr2 = self.data[i+1] as char;
			if chr1 == '\r' && chr2 == '\n' {
				return Some(i);
			}
		}
		None
	}
	
	/// Finds '\n' in buffer
	pub fn find_eol(&self) -> Option<usize> {
		self.do_find_eol(self.read_index, self.write_index)
	}
	
	pub fn find_eol_from(&self, start: usize) -> Option<usize> {
		assert!(self.read_index <= start);
		assert!(start <= self.write_index);
		self.do_find_eol(start, self.write_index)
	}
	
	fn do_find_eol(&self, start: usize, end: usize) -> Option<usize> {
		assert!(start <= end);
		for i in start..end {
			let chr = self.data[i] as char;
			if chr == '\n' {
				return Some(i);
			}
		}
		None
	}
	
	/// Peeks one byte in buffer
	pub fn peek(&self) -> *const u8 { &self.data[self.read_index] }
	
	/// Peeks an int64 in buffer
	pub fn peek_int64(&self) -> i64 {
		assert!(self.readable_bytes() >= mem::size_of::<i64>());
		let mut bytes: [u8; 8] = [0u8; 8];
		let be64: i64;
		unsafe { 
			ptr::copy_nonoverlapping(self.peek(), &mut bytes[0], mem::size_of::<i64>());
			be64 = mem::transmute::<[u8; 8], i64>(bytes);
		}
		i64::from_be(be64)
	}
	
	/// Peeks an int32 in buffer
	pub fn peek_int32(&self) -> i32 {
		assert!(self.readable_bytes() >= mem::size_of::<i32>());
		let mut bytes: [u8; 4] = [0u8; 4];
		let be32: i32;
		unsafe {
			ptr::copy_nonoverlapping(self.peek(), &mut bytes[0], mem::size_of::<i32>());
			be32 = mem::transmute::<[u8; 4], i32>(bytes);
		}
		i32::from_be(be32)
	}
	
	/// Peeks an int16 in buffer
	pub fn peek_int16(&self) -> i16 {
		assert!(self.readable_bytes() >= mem::size_of::<i16>());
		let mut bytes: [u8; 2] = [0u8; 2];
		let be16: i16;
		unsafe { 
			ptr::copy_nonoverlapping(self.peek(), &mut bytes[0], mem::size_of::<i16>());
			be16 = mem::transmute::<[u8; 2], i16>(bytes);
		}
		i16::from_be(be16)
	}
	
	/// Peeks an int8 in buffer
	pub fn peek_int8(&self) -> i8 {
		assert!(self.readable_bytes() >= mem::size_of::<i8>());
		let be8: i8;
		unsafe { be8 = *self.peek() as i8; }
		i8::from_be(be8)
	}
	
	/// Appends bytes in buffer
	pub fn append_bytes(&mut self, bytes: &[u8]) {
		self.ensure_writable_bytes(bytes.len());
		unsafe { ptr::copy_nonoverlapping(&bytes[0], self.begin_write(), bytes.len()); }
		self.has_written(bytes.len());
	}
	
	/// Appends a string in buffer
	pub fn append_string(&mut self, str: &String) { self.append_bytes(str.as_bytes()); }
	
	/// Appends int64 using network endian
	pub fn append_int64(&mut self, x: i64) {
		let be64 = x.to_be();
		let bytes: [u8; 8] = unsafe {
			mem::transmute::<i64, [u8; 8]>(be64)
		};
		self.append_bytes(&bytes);
	}
	
	/// Appends int32 using network endian
	pub fn append_int32(&mut self, x: i32) {
		let be32 = x.to_be();
		let bytes: [u8; 4] = unsafe {
			mem::transmute::<i32, [u8; 4]>(be32)
		};
		self.append_bytes(&bytes);
	}
	
	/// Appends int16 using network endian
	pub fn append_int16(&mut self, x: i16) {
		let be16 = x.to_be();
		let bytes: [u8; 2] = unsafe {
			mem::transmute::<i16, [u8; 2]>(be16)
		};
		self.append_bytes(&bytes);
	}
	
	/// Appends int8 in buffer
	pub fn append_int8(&mut self, x: i8) {
		let bytes: [u8; 1] = unsafe {
			mem::transmute::<i8, [u8; 1]>(x)
		};
		self.append_bytes(&bytes);
	}
	
	/// Ensures 'len' bytes space left
	pub fn ensure_writable_bytes(&mut self, len: usize) {
		if self.writable_bytes() < len {
			self.make_space(len);
		}
		assert!(self.writable_bytes() >= len);
	}
	
	/// Writes 'len' bytes data
	pub fn has_written(&mut self, len: usize) {
		assert!(len <= self.writable_bytes());
		self.write_index += len;
	}
	
	/// Makes enough space in buffer
	pub fn make_space(&mut self, len: usize) {
		if self.writable_bytes() + self.prependable_bytes() < PREPEND + len {
			self.data.resize(self.write_index + len, 0u8);
		} else {
			assert!(PREPEND < self.read_index);
			let readable = self.readable_bytes();
			unsafe { ptr::copy(&self.data[self.read_index], &mut self.data[PREPEND], readable) };
			self.read_index = PREPEND;
			self.write_index = self.read_index + readable;
			assert!(readable == self.readable_bytes());
		}
	}
	
	/// Retrieves bytes in buffer as string
	pub fn retrieve_as_string(&mut self, len: usize) -> String {
		assert!(len <= self.readable_bytes());
		let mut bytes = vec![0u8; len];
		unsafe { ptr::copy_nonoverlapping(self.peek(), &mut bytes[0], len); }
		self.retrieve(len);
		String::from_utf8(bytes).unwrap()
	}
	
	/// Retrieves 'len' bytes in buffer
	pub fn retrieve(&mut self, len: usize) {
		assert!(len <= self.readable_bytes());
		if len < self.readable_bytes() {
			self.read_index += len;
		} else {
			self.retrieve_all();
		}
	}
	
	/// Retrieves int64 in buffer
	pub fn retrieve_int64(&mut self) { self.retrieve(mem::size_of::<i64>()) }
	
	/// Retrieves int32 in buffer
	pub fn retrieve_int32(&mut self) { self.retrieve(mem::size_of::<i32>()) }
	
	/// Retrieves int16 in buffer
	pub fn retrieve_int16(&mut self) { self.retrieve(mem::size_of::<i16>()) }
	
	/// Retrieves int8 in buffer
	pub fn retrieve_int8(&mut self) { self.retrieve(mem::size_of::<i8>()) }
	
	/// Retrieves all bytes in buffer
	pub fn retrieve_all(&mut self) {
		self.read_index = PREPEND;
		self.write_index = PREPEND;
	}
	
	/// Retrieves util the 'end' position
	pub fn retrieve_until(&mut self, end: usize) {
		assert!(self.read_index <= end);
		assert!(end <= self.write_index);
		let rindex = self.read_index;
		self.retrieve(end - rindex);
	}
	
	/// Retrieves all bytes as string
	pub fn retrieve_all_as_string(&mut self) -> String {
		let readable = self.readable_bytes();
		self.retrieve_as_string(readable)
	}
	
	/// Prepends bytes in buffer
	pub fn prepend_bytes(&mut self, bytes: &[u8]) {
		assert!(bytes.len() <= self.prependable_bytes());
		self.read_index -= bytes.len();
		unsafe { ptr::copy_nonoverlapping(&bytes[0], &mut self.data[self.read_index], bytes.len()); }
	}
	
	/// Prepends an int64 in buffer using network endian 
	pub fn prepend_int64(&mut self, x: i64) {
		let be64 = x.to_be();
		let bytes: [u8; 8] = unsafe { mem::transmute::<i64, [u8; 8]>(be64) };
		self.prepend_bytes(&bytes);
	}
	
	/// Prepends an int32 in buffer using network endian 
	pub fn prepend_int32(&mut self, x: i32) {
		let be32 = x.to_be();
		let bytes: [u8; 4] = unsafe { mem::transmute::<i32, [u8; 4]>(be32) };
		self.prepend_bytes(&bytes);
	}
	
	/// Prepends an int16 in buffer using network endian 
	pub fn prepend_int16(&mut self, x: i16) {
		let be16 = x.to_be();
		let bytes: [u8; 2] = unsafe { mem::transmute::<i16, [u8; 2]>(be16) };
		self.prepend_bytes(&bytes);
	}
	
	/// Prepends an int8 in buffer
	pub fn prepend_int8(&mut self, x: i8) {
		let bytes: [u8; 1] = unsafe { mem::transmute::<i8, [u8; 1]>(x) };
		self.prepend_bytes(&bytes);
	}
	
	pub fn unwrite(&mut self, len: usize) {
		assert!(len <= self.readable_bytes());
		self.write_index -= len;
	}
	
	pub fn internal_capacity(&self) -> usize { self.data.capacity() }
	
	/// Read int64 from network endian
	pub fn read_int64(&mut self) -> i64 {
		let ret = self.peek_int64();
		self.retrieve_int64();
		ret
	}
	
	/// Read int32 from network endian
	pub fn read_int32(&mut self) -> i32 {
		let ret = self.peek_int32();
		self.retrieve_int32();
		ret
	}
	
	/// Read int16 from network endian
	pub fn read_int16(&mut self) -> i16 {
		let ret = self.peek_int16();
		self.retrieve_int16();
		ret
	}
	
	/// Read int8 from network endian
	pub fn read_int8(&mut self) -> i8 {
		let ret = self.peek_int8();
		self.retrieve_int8();
		ret
	}
	
	pub fn begin_write(&mut self) -> *mut u8 { &mut self.data[self.write_index] }
	
	/// Reads from stream
	pub fn read_from(&mut self, stream: &mut Read) -> Result<usize> {
		let mut bytes = [0u8; 65536];
		let received = try!(stream.read(&mut bytes));
		self.append_bytes(&bytes[..received]);
		Ok(received)
	}
}

mod tests {
	#[allow(unused_imports)]
	use super::*;
	#[allow(unused_imports)]
	use std::mem;
	
	#[test]
	fn test_append_retrieve() {
		let mut buf: Buffer = Buffer::new(None);
		assert_eq!(buf.readable_bytes(), 0);
		assert_eq!(buf.writable_bytes(), INITIAL);
		assert_eq!(buf.prependable_bytes(), PREPEND);
		
		let mut string = String::new();
		for _ in 0..200 {
			string.push('x');
		}
		buf.append_string(&string);
		assert_eq!(buf.readable_bytes(), string.len());
		assert_eq!(buf.writable_bytes(), INITIAL - string.len());
		assert_eq!(buf.prependable_bytes(), PREPEND);
		
		let string2 = buf.retrieve_as_string(50);
		assert_eq!(string2.len(), 50);
		assert_eq!(buf.readable_bytes(), string.len() - string2.len());
		assert_eq!(buf.writable_bytes(), INITIAL - string.len());
		assert_eq!(buf.prependable_bytes(), PREPEND + string2.len());
		
		buf.append_string(&string);
		assert_eq!(buf.readable_bytes(), 2 * string.len() - string2.len());
		assert_eq!(buf.writable_bytes(), INITIAL - 2 * string.len());
		assert_eq!(buf.prependable_bytes(), PREPEND + string2.len());
		
		let string3 = buf.retrieve_all_as_string();
		assert_eq!(string3.len(), 350);
		assert_eq!(buf.readable_bytes(), 0);
		assert_eq!(buf.writable_bytes(), INITIAL);
		assert_eq!(buf.prependable_bytes(), PREPEND);
	}
	
	#[test]
	fn test_buffer_grow() {
		let mut string = String::new();
		for _ in 0..400 {
			string.push('y');
		}
		let mut buf: Buffer = Buffer::new(None);
		buf.append_string(&string);
		assert_eq!(buf.readable_bytes(), 400);
		assert_eq!(buf.writable_bytes(), INITIAL - 400);
		
		buf.retrieve(50);
		assert_eq!(buf.readable_bytes(), 350);
		assert_eq!(buf.writable_bytes(), INITIAL - 400);
		assert_eq!(buf.prependable_bytes(), PREPEND + 50);
		
		let mut string2 = String::new();
		for _ in 0..1000 {
			string2.push('z');
		}
		buf.append_string(&string2);
		assert_eq!(buf.readable_bytes(), 1350);
		assert_eq!(buf.writable_bytes(), 0);
		assert_eq!(buf.prependable_bytes(), PREPEND + 50);
		
		buf.retrieve_all();
		assert_eq!(buf.readable_bytes(), 0);
		assert_eq!(buf.writable_bytes(), 1400);
		assert_eq!(buf.prependable_bytes(), PREPEND);
	}
	
	#[test]
	fn test_buffer_inside_grow() {
		let mut string = String::new();
		for _ in 0..800 {
			string.push('y');
		}
		let mut buf = Buffer::new(None);
		buf.append_string(&string);
		assert_eq!(buf.readable_bytes(), 800);
		assert_eq!(buf.writable_bytes(), INITIAL - 800);
		
		buf.retrieve(500);
		assert_eq!(buf.readable_bytes(), 300);
		assert_eq!(buf.writable_bytes(), INITIAL - 800);
		assert_eq!(buf.prependable_bytes(), PREPEND + 500);
		
		let mut string2 = String::new();
		for _ in 0..300 {
			string2.push('z');
		}
		buf.append_string(&string2);
		assert_eq!(buf.readable_bytes(), 600);
		assert_eq!(buf.writable_bytes(), INITIAL - 600);
		assert_eq!(buf.prependable_bytes(), PREPEND);
	}
	
	#[test]
	fn test_buffer_shrink() {
		let mut string = String::new();
		for _ in 0..2000 {
			string.push('y');
		}
		let mut buf = Buffer::new(None);
		buf.append_string(&string);
		assert_eq!(buf.readable_bytes(), 2000);
		assert_eq!(buf.writable_bytes(), 0);
		assert_eq!(buf.prependable_bytes(), PREPEND);
		
		buf.retrieve(1500);
		assert_eq!(buf.readable_bytes(), 500);
		assert_eq!(buf.writable_bytes(), 0);
		assert_eq!(buf.prependable_bytes(), PREPEND + 1500);
		
		let mut string2 = String::new();
		for _ in 0..500 {
			string2.push('y');
		}
		buf.shrink(0);
		assert_eq!(buf.readable_bytes(), 500);
		assert_eq!(buf.writable_bytes(), INITIAL - 500);
		assert_eq!(buf.retrieve_all_as_string(), string2);
		assert_eq!(buf.prependable_bytes(), PREPEND);
	}
	
	#[test]
	fn test_buffer_prepend() {
		let mut string = String::new();
		for _ in 0..200 {
			string.push('y');
		}
		let mut buf: Buffer = Buffer::new(None);
		buf.append_string(&string);
		assert_eq!(buf.readable_bytes(), 200);
		assert_eq!(buf.writable_bytes(), INITIAL - 200);
		assert_eq!(buf.prependable_bytes(), PREPEND);
		
		let x: i32 = 0;
		let bytes: [u8; 4] = unsafe { mem::transmute::<i32, [u8; 4]>(x) };
		buf.prepend_bytes(&bytes);
		assert_eq!(buf.readable_bytes(), 204);
		assert_eq!(buf.writable_bytes(), INITIAL - 200);
		assert_eq!(buf.prependable_bytes(), PREPEND - 4);
	}
	
	#[test]
	fn test_buffer_read_int() {
		let mut buf: Buffer = Buffer::new(None);
		buf.append_string(&"HTTP".to_string());
		assert_eq!(buf.readable_bytes(), 4);
		assert_eq!(buf.peek_int8(), 'H' as i8);
		let top16 = buf.peek_int16();
		assert_eq!(top16, 'H' as i16 * 256 + 'T' as i16);
		assert_eq!(buf.peek_int32(), top16 as i32 * 65536 + 'T' as i32 * 256 + 'P' as i32);
		
		assert_eq!(buf.read_int8(), 'H' as i8);
		assert_eq!(buf.read_int16(), 'T' as i16 * 256 + 'T' as i16);
		assert_eq!(buf.read_int8(), 'P' as i8);
		assert_eq!(buf.readable_bytes(), 0);
		assert_eq!(buf.writable_bytes(), INITIAL);
		
		buf.append_int8(-1);
		buf.append_int16(-1);
		buf.append_int32(-1);
		assert_eq!(buf.readable_bytes(), 7);
		assert_eq!(buf.read_int8(), -1);
		assert_eq!(buf.read_int32(), -1);
		assert_eq!(buf.read_int16(), -1);
	}
	
	#[test]
	fn test_buffer_find_eol() {
		let mut string = String::new();
		for _ in 0..100_000 {
			string.push('x');
		}
		let mut buf: Buffer = Buffer::new(None);
		buf.append_string(&string);
		assert_eq!(buf.find_eol(), None);
		assert_eq!(buf.find_eol_from(90_000), None);
	}
	
	#[test]
	fn test_buffer_find_crlf() {
		let mut string = String::new();
		for _ in 0..100_000 {
			string.push('x');
		}
		let mut buf: Buffer = Buffer::new(None);
		buf.append_string(&string);
		assert_eq!(buf.find_crlf(), None);
//		assert_eq!(buf.find_crlf_from(90_000), None);
	}
}