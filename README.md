# Buffer

Buffer is an application-layer byte buffer designed to be used in asynchronous network applications.

Here is how you read first a single bit, then three bits and finally four bits from a byte buffer:

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

You can find more examples in the unit test code.

This is a byte buffer class modeled after muduo::net::Buffer

+-------------------+------------------+------------------+
| prependable bytes |  readable bytes  |  writable bytes  |
|                   |     (CONTENT)    |                  |
+-------------------+------------------+------------------+
|                   |                  |                  |
0      <=      readerIndex   <=   writerIndex    <=     size
