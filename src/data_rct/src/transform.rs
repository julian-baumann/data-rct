pub fn get_message_part(raw_content: &mut Vec<u8>) -> Option<Vec<u8>> {
    let mut index: usize = 0;
    let mut result: Vec<u8> = Vec::new();

    for byte in raw_content.iter() {
        index += 1;

        if PartialEq::eq(byte, &0u8) {
            raw_content.drain(..index);

            return Some(result);
        }

        result.push(byte.to_owned());
    }

    return None;
}

pub fn get_utf8_message_part(raw_content: &mut Vec<u8>) -> Option<String> {
    let message = get_message_part(raw_content)?;
    let result = String::from_utf8(message);

    if let Ok(result) = result {
        return Some(result);
    }

    return None;
}

pub trait ByteConvertable {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(message: &mut Vec<u8>, ip_address: String) -> Option<Self> where Self: Sized;
}