#[derive(PartialEq, Debug)]
#[repr(C)]
pub enum PluginResult {
    Success,
    NoChain,
    NoChainAndCancel,
}

#[repr(C)]
pub struct EncodedString {
    ptr: *const u8,
    len: usize,
}
impl EncodedString {
    pub fn new(ptr: *const u8, len: usize) -> Self {
        EncodedString { ptr: ptr, len: len }
    }
    pub fn to_vec(&self) -> Vec<u8> {
        let slice = unsafe { std::slice::from_raw_parts(self.ptr, self.len) };
        slice.to_owned()
    }
    pub fn to_string(&self) -> Result<String, std::string::FromUtf8Error> {
        let slice = unsafe { std::slice::from_raw_parts(self.ptr, self.len) };
        String::from_utf8(slice.to_owned())
    }
}