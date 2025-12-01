use std::fmt::Display;

/// Wrapper for `loro::FractionalIndex`
#[derive(Debug, Clone)]
pub struct FractionalIndex {
    inner: loro::FractionalIndex,
}

impl FractionalIndex {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            inner: loro::FractionalIndex::from_bytes(bytes),
        }
    }

    pub fn from_hex_string(str: &str) -> Self {
        Self {
            inner: loro::FractionalIndex::from_hex_string(str),
        }
    }
}

impl Display for FractionalIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.to_string())
    }
}

impl From<loro::FractionalIndex> for FractionalIndex {
    fn from(inner: loro::FractionalIndex) -> Self {
        Self { inner }
    }
}

impl From<FractionalIndex> for loro::FractionalIndex {
    fn from(wrapper: FractionalIndex) -> Self {
        wrapper.inner
    }
}
