#[allow(unused_imports)]
use super::*;
///A buffer is a binary blob. It is either the binary chunk of the subtree file, or an external buffer referenced by a URI.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Buffer {
    ///The length of the buffer in bytes.
    pub byte_length: usize,
    ///The name of the buffer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Box<str>>,
    ///The URI (or IRI) of the file that contains the binary buffer data. Relative paths are relative to the file containing the buffer JSON. `uri` is required when using the JSON subtree format and not required when using the binary subtree format - when omitted the buffer refers to the binary chunk of the subtree file. Data URIs are not allowed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Box<str>>,
    ///The buffer's decoded bytes; never part of the JSON.
    #[serde(skip)]
    pub data: bytes::Bytes,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for Buffer {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
///A contiguous subset of a buffer
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferView {
    ///The index of the buffer.
    pub buffer: usize,
    ///The total byte length of the buffer view.
    pub byte_length: usize,
    ///The offset into the buffer in bytes.
    pub byte_offset: usize,
    ///The name of the `bufferView`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Box<str>>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for BufferView {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
