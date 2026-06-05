use core::{error, fmt, str};

use crate::core::decoding::modules::sections::SectionTy;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DecodingError {
    /// The magic number at the start of the Wasm bytecode is invalid.
    InvalidMagic,
    /// The binary format version at the start of the Wasm bytecode is invalid.
    InvalidBinaryFormatVersion,
    /// The end of the binary file was reached unexpectedly.
    Eof,

    /// A UTF-8 string is malformed.
    MalformedUtf8(str::Utf8Error),
    /// The type of a section is malformed.
    MalformedSectionTypeDiscriminator(u8),
    /// The discriminator of a number type is malformed.
    MalformedNumTypeDiscriminator(u8),
    /// The discriminator of a vector type is malformed.
    MalformedVecTypeDiscriminator(u8),
    /// The discriminator of a function type is malformed.
    MalformedFuncTypeDiscriminator(u8),
    /// The discriminator of a reference type is malformed.
    MalformedRefTypeDiscriminator(u8),
    /// A valtype is malformed because it is neither a number, reference nor vector type.
    MalformedValType,
    /// The discriminator of an export description is malformed.
    MalformedExportDescDiscriminator(u8),
    /// The discriminator of an import description is malformed.
    MalformedImportDescDiscriminator(u8),
    /// The discriminator of a limits type is malformed.
    MalformedLimitsDiscriminator(u8),
    /// The discriminator of a mut type is malformed.
    MalformedMutDiscriminator(u8),
    /// Block types use a special 33-bit signed integer for encoding type indices.
    MalformedBlockTypeTypeIdx(i64),
    /// A variable-length integer was read but it overflowed.
    MalformedVariableLengthInteger,
    /// The discriminator of an element kind is malformed.
    MalformedElemKindDiscriminator(u8),
    /// 33-bit signed integers are sometimes used to encode unsigned 32-bit
    /// integers to prevent collisions between bit patterns of different types.
    /// Therefore, 33-bit signed integers may never be negative.
    I33IsNegative,
    /// A function specifies too many locals, i.e. more than 2^32 - 1
    TooManyLocals,
    /// A section's contents were successfully decoded, but either too many or not enough bytes of
    /// the section's bytecode were consumed.
    SectionSizeMismatch,
    /// A section with given type is out of order. All section types have a fixed order in which they must occur.
    SectionOutOfOrder(SectionTy),
    /// A table type was marked as shared but shared tables are not yet
    /// implemented.
    SharedTablesNotYetImplemented,
    /// A memory type is shared and thus requires a max limit but none was set.
    SharedMemoryWithoutMaxLimit,
}

impl error::Error for DecodingError {}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodingError::InvalidMagic => write!(f, "The magic number is invalid"),
            DecodingError::InvalidBinaryFormatVersion => write!(f, "The Wasm binary format version is invalid"),
            DecodingError::Eof => write!(f, "The end of the Wasm bytecode was reached unexpectedly"),
            DecodingError::MalformedUtf8(utf8_error) => write!(f, "Failed to parse a UTF-8 string: {utf8_error}"),
            DecodingError::MalformedSectionTypeDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as a section type discriminator"),
            DecodingError::MalformedNumTypeDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as a number type discriminator"),
            DecodingError::MalformedVecTypeDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as a vector type discriminator"),
            DecodingError::MalformedFuncTypeDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as a function type discriminator"),
            DecodingError::MalformedRefTypeDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as a reference type discriminator"),
            DecodingError::MalformedValType => write!(f, "Failed to read a value type because it is neither a number, reference or vector type"),
            DecodingError::MalformedExportDescDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as an export description discriminator"),
            DecodingError::MalformedImportDescDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as an import description discriminator"),
            DecodingError::MalformedLimitsDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as a limits type discriminator"),
            DecodingError::MalformedMutDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as a mute type discriminator"),
            DecodingError::MalformedBlockTypeTypeIdx(idx) => write!(f, "The type index {idx} which is encoded as a singed 33-bit integer inside a block type is malformed"),
            DecodingError::MalformedVariableLengthInteger => write!(f, "Reading a variable-length integer overflowed"),
            DecodingError::MalformedElemKindDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as an element kind discriminator"),
            DecodingError::I33IsNegative => f.write_str("An i33 type is negative which is not allowed"),
            DecodingError::TooManyLocals => write!(f,"A function specifies too many locals, i.e. more than of 2^32-1"),
            DecodingError::SectionSizeMismatch => f.write_str("A section's contents were successfully decoded, but either too many or not enough bytes of the section's bytecode were consumed."),
            DecodingError::SectionOutOfOrder(ty) => write!(f, "A section of type `{ty:?}` is defined out of order"),
            DecodingError::SharedTablesNotYetImplemented => f.write_str("A table type was marked as shared but shared tables are not yet implemented"),
            DecodingError::SharedMemoryWithoutMaxLimit => f.write_str("A memory type is shared and thus requires a max limit but none was set"),
        }
    }
}
