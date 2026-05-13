pub trait ByteUtils {
    fn to_lossy_string(&self) -> String;

    fn to_u16(&self) -> u16;
}

impl ByteUtils for [u8] {
    fn to_lossy_string(&self) -> String {
        String::from_utf8_lossy(self).into_owned()
    }
    fn to_u16(&self) -> u16 {
        let bytes: [u8; 2] = self.try_into().expect("Slice with incorrect length");
        u16::from_be_bytes(bytes)
    }
}
