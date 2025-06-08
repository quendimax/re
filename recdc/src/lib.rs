mod codec;
pub use codec::Codec;

mod error;
pub use error::CodecError;

mod utf8;
pub use utf8::Utf8Codec;
