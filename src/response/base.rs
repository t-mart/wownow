use std::fmt::Display;

/// Errors that can occur when parsing a response.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An unknown type name was encountered.
    #[error("unknown type name `{0}`")]
    UnknownTypeName(String),

    /// A type should have a colon separating the type name and length
    #[error("type `{0}` should have colon separator")]
    ExpectedColon(String),

    /// A type should have a bang separating the name and type
    #[error("type `{0}` should have a bang separator")]
    ExpectedBang(String),

    /// A response should have exactly one seqn line
    #[error("response should have exactly one seqn line")]
    MultipleSeqn,

    /// A record should have exactly as many fields as the header
    #[error("record field length ({0}) should match header field length ({1})")]
    MismatchedRecordLength(usize, usize),

    /// A response should have header line
    #[error("response should have header line")]
    ExpectedHeaderLine,

    /// A response should have seqn line
    #[error("response should have seqn line")]
    ExpectedSeqnLine,

    /// A record should have field with the given name
    #[error("record should have field `{0}`")]
    ExpectedField(&'static str),

    /// A field should be non-empty
    #[error("field {0} should be non-empty")]
    EmptyField(String),

    /// A field should be of the expected type
    #[error("Cannot deserialize type `{0}` as type `{1}`")]
    UnexpectedType(String, String),

    /// An error occurred while parsing an integer
    #[error(transparent)]
    UnparseableInt(#[from] std::num::ParseIntError),

    /// An error occurred interpreting bytes as UTF-8
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum TypeName {
    /// A type for string data. E.g,. "STRING:0". Strings always seem to have a dummy length of 0,
    /// so presumably unbounded length. Although I haven't seen it, this could be an ASCII string.
    /// Doesn't seem to hurt to support that.
    String,

    /// A type for n pairs of hex chars (0-255). E.g., "HEX:16".
    Hex,

    /// A type for n "decimal" bytes, e.g., "DEC:4".
    ///
    /// It's not clear what this type really models here, but it seems to be a base-10 (hence
    /// "decimal") integer. I do feel that these are not numbers with fractional parts: I'd expect
    /// "FLOAT" for that, in convention with computing terminology. Further, maybe it's signed, but
    /// I've never encountered such a value from testing, so we'll consider these unsigned until one
    /// presents itself.
    Dec,
}

impl<'input> TryFrom<&'input str> for TypeName {
    type Error = Error;

    fn try_from(name: &'input str) -> Result<Self> {
        match name.to_uppercase().as_ref() {
            "STRING" => Ok(TypeName::String),
            "HEX" => Ok(TypeName::Hex),
            "DEC" => Ok(TypeName::Dec),
            _ => Err(Error::UnknownTypeName(name.to_owned())),
        }
    }
}

impl Display for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeName::String => write!(f, "STRING"),
            TypeName::Hex => write!(f, "HEX"),
            TypeName::Dec => write!(f, "DEC"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) struct Type {
    name: TypeName,
    length: usize,
}

impl<'input> TryFrom<&'input str> for Type {
    type Error = Error;

    fn try_from(type_: &str) -> Result<Self> {
        let Some((name, length)) = type_.split_once(':') else {
            return Err(Error::ExpectedColon(type_.to_owned()));
        };

        let length = length.parse::<usize>()?;

        Ok(Self {
            name: name.try_into()?,
            length,
        })
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.name, self.length)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Header<'input> {
    name: &'input str,
    type_: Type,
}

impl<'input> Header<'input> {
    fn parse_header_line(s: &'input str) -> Result<Vec<Self>> {
        s.split('|').map(Self::try_from).collect()
    }
}

impl<'input> TryFrom<&'input str> for Header<'input> {
    type Error = Error;

    fn try_from(s: &'input str) -> Result<Self> {
        if let Some((name, tact_type)) = s.split_once('!') {
            Ok(Self {
                name,
                type_: tact_type.try_into()?,
            })
        } else {
            Err(Error::ExpectedBang(s.to_owned()))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Response<'input> {
    /// A monotonically increasing identifier. Used by caches to determine if they need to update.
    pub(super) seqn: u32,

    headers: Vec<Header<'input>>,

    records: Vec<Vec<&'input str>>,
}

impl<'input> Response<'input> {
    pub(super) fn iter_records(&self) -> impl Iterator<Item = Record<'_, 'input>> {
        self.records.iter().map(move |values| Record {
            headers: &self.headers,
            values,
        })
    }
}

impl<'input> TryFrom<&'input str> for Response<'input> {
    type Error = Error;

    fn try_from(input: &'input str) -> Result<Self> {
        let mut headers: Option<Vec<Header>> = None;
        let mut seqn: Option<u32> = None;
        let mut records: Vec<Vec<&'input str>> = Vec::new();

        for line in input.lines() {
            if let Some(number_part) = line.strip_prefix("## seqn = ") {
                if seqn.is_some() {
                    return Err(Error::MultipleSeqn);
                }
                seqn = Some(number_part.parse()?);
            } else if headers.is_none() {
                headers = Some(Header::parse_header_line(line)?);
            } else {
                // unwrapping is safe because we've already parsed the header line
                let headers_len = headers.as_ref().unwrap().len();

                let record = line.split('|').collect::<Vec<_>>();
                if record.len() != headers_len {
                    return Err(Error::MismatchedRecordLength(record.len(), headers_len));
                }
                records.push(record);
            }
        }

        let headers = headers.ok_or(Error::ExpectedHeaderLine)?;
        let seqn = seqn.ok_or(Error::ExpectedSeqnLine)?;

        Ok(Self {
            seqn,
            headers,
            records,
        })
    }
}

impl<'input> TryFrom<&'input [u8]> for Response<'input> {
    type Error = Error;

    fn try_from(input: &'input [u8]) -> Result<Self> {
        let input = std::str::from_utf8(input)?;
        Self::try_from(input)
    }
}

pub(super) struct Field<'resp, 'input> {
    type_: &'resp Type,
    value: &'resp &'input str,
}

const STRING_TYPE: Type = Type {
    name: TypeName::String,
    length: 0,
};
const HEX16_TYPE: Type = Type {
    name: TypeName::Hex,
    length: 16,
};
const DEC4_TYPE: Type = Type {
    name: TypeName::Dec,
    length: 4,
};

/// A field with type "STRING:0", a String
pub type String0 = String;

/// A field with type "HEX:16", a 16-byte value
pub type Hex16 = [u8; 16];

/// A field with type "DEC:4", a u32
pub type Dec4 = u32;

impl<'resp, 'input> TryFrom<&Field<'resp, 'input>> for Option<Hex16> {
    type Error = Error;

    fn try_from(field: &Field<'resp, 'input>) -> Result<Self> {
        if field.type_ != &HEX16_TYPE {
            return Err(Error::UnexpectedType(
                field.type_.to_string(),
                HEX16_TYPE.to_string(),
            ));
        }
        if field.value.is_empty() {
            Ok(None)
        } else {
            let mut bytes = [0; 16];
            let mut iter = field.value.as_bytes().chunks(2);
            for (idx, chunk) in iter.by_ref().enumerate() {
                bytes[idx] =
                    u8::from_str_radix(unsafe { std::str::from_utf8_unchecked(chunk) }, 16)?;
            }
            Ok(Some(bytes))
        }
    }
}

impl<'resp, 'input> TryFrom<&Field<'resp, 'input>> for Hex16 {
    type Error = Error;

    fn try_from(field: &Field<'resp, 'input>) -> Result<Self> {
        let opt = <Option<[u8; 16]>>::try_from(field)?;
        opt.ok_or(Error::EmptyField(field.type_.to_string()))
    }
}

impl<'resp, 'input> TryFrom<&Field<'resp, 'input>> for Dec4 {
    type Error = Error;

    fn try_from(field: &Field<'resp, 'input>) -> Result<Self> {
        if field.type_ != &DEC4_TYPE {
            return Err(Error::UnexpectedType(
                field.type_.to_string(),
                DEC4_TYPE.to_string(),
            ));
        }
        Ok(field.value.parse::<u32>()?)
    }
}

impl<'resp, 'input> TryFrom<&Field<'resp, 'input>> for String0 {
    type Error = Error;

    fn try_from(field: &Field<'resp, 'input>) -> Result<Self> {
        if field.type_ != &STRING_TYPE {
            return Err(Error::UnexpectedType(
                field.type_.to_string(),
                DEC4_TYPE.to_string(),
            ));
        }
        Ok((*field.value).to_string())
    }
}

pub(super) struct Record<'resp, 'input> {
    headers: &'resp Vec<Header<'input>>,
    values: &'resp Vec<&'input str>,
}

impl<'resp, 'input> Record<'resp, 'input> {
    pub(super) fn get_field_by_header_name(&self, name: &str) -> Option<Field<'resp, 'input>> {
        self.headers.iter().enumerate().find_map(|(idx, header)| {
            if header.name == name {
                Some(Field {
                    type_: &header.type_,
                    value: &self.values[idx],
                })
            } else {
                None
            }
        })
    }
}
