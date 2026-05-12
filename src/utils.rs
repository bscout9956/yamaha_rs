pub fn bytes_to_str(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}