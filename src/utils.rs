pub trait ByteUtils {
    fn to_lossy_string(&self) -> String;
}

impl ByteUtils for [u8] {
    fn to_lossy_string(&self) -> String {
        String::from_utf8_lossy(self).into_owned()
    }
}