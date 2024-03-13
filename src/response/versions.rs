//! Model for the versions response
use crate::response::base::{
    Dec4, Error, Hex16, Record as BaseRecord, Response as BaseResponse, Result, String0,
};

/// A record in the versions response
#[derive(Debug, PartialEq)]
pub struct Record {
    /// The region
    pub region: String0,

    /// The build config
    pub build_config: Hex16,

    /// The CDN config
    pub cdn_config: Hex16,

    /// The key ring
    // This is always an empty string I've seen, but allow it to be its declared type
    pub key_ring: Option<Hex16>,

    /// The build ID
    pub build_id: Dec4,

    /// The versions name
    pub versions_name: String0,

    /// The product config
    pub product_config: Hex16,
}

impl<'input> TryFrom<BaseRecord<'_, 'input>> for Record {
    type Error = Error;

    fn try_from(record: BaseRecord<'_, 'input>) -> Result<Self> {
        let region = (&record
            .get_field_by_header_name("Region")
            .ok_or(Error::ExpectedField("Region"))?)
            .try_into()?;
        let build_config = (&record
            .get_field_by_header_name("BuildConfig")
            .ok_or(Error::ExpectedField("BuildConfig"))?)
            .try_into()?;
        let cdn_config = (&record
            .get_field_by_header_name("CDNConfig")
            .ok_or(Error::ExpectedField("CDNConfig"))?)
            .try_into()?;
        let key_ring = (&record
            .get_field_by_header_name("KeyRing")
            .ok_or(Error::ExpectedField("KeyRing"))?)
            .try_into()?;
        let build_id = (&record
            .get_field_by_header_name("BuildId")
            .ok_or(Error::ExpectedField("BuildId"))?)
            .try_into()?;
        let versions_name = (&record
            .get_field_by_header_name("VersionsName")
            .ok_or(Error::ExpectedField("VersionsName"))?)
            .try_into()?;
        let product_config = (&record
            .get_field_by_header_name("ProductConfig")
            .ok_or(Error::ExpectedField("ProductConfig"))?)
            .try_into()?;

        Ok(Self {
            region,
            build_config,
            cdn_config,
            key_ring,
            build_id,
            versions_name,
            product_config,
        })
    }
}

/// The versions response
#[derive(Debug, PartialEq)]
pub struct Response {
    /// The sequence number
    pub seqn: u32,

    /// The records
    pub records: Vec<Record>,
}

impl<'input> TryFrom<BaseResponse<'input>> for Response {
    type Error = Error;

    fn try_from(response: BaseResponse<'input>) -> Result<Self> {
        Ok(Self {
            seqn: response.seqn,
            records: response
                .iter_records()
                .map(TryInto::try_into)
                .collect::<Result<_>>()?,
        })
    }
}

impl<'input> TryFrom<&'input str> for Response {
    type Error = Error;

    fn try_from(input: &'input str) -> Result<Self> {
        let response = BaseResponse::try_from(input)?;
        Response::try_from(response)
    }
}

impl<'input> TryFrom<&'input [u8]> for Response {
    type Error = Error;

    fn try_from(input: &'input [u8]) -> Result<Self> {
        let response = BaseResponse::try_from(input)?;
        Response::try_from(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versions_response() {
        let input = "Region!STRING:0|BuildConfig!HEX:16|CDNConfig!HEX:16|KeyRing!HEX:16|BuildId!DEC:4|VersionsName!String:0|ProductConfig!HEX:16\n\
        ## seqn = 2118468\n\
        us|47e9e06f8371afb141e22614a912acc8|74093d42ce367c7a67f2831dbf64088d||53584|10.2.5.53584|53020d32e1a25648c8e1eafd5771935f\n\
        eu|47e9e06f8371afb141e22614a912acc8|74093d42ce367c7a67f2831dbf64088d||53584|10.2.5.53584|53020d32e1a25648c8e1eafd5771935f";

        let versions_response = Response::try_from(input).unwrap();

        assert_eq!(
            versions_response,
            Response {
                seqn: 2_118_468,
                records: vec![
                    Record {
                        region: "us".to_owned(),
                        build_config: [
                            0x47, 0xe9, 0xe0, 0x6f, 0x83, 0x71, 0xaf, 0xb1, 0x41, 0xe2, 0x26, 0x14,
                            0xa9, 0x12, 0xac, 0xc8
                        ],
                        cdn_config: [
                            0x74, 0x09, 0x3d, 0x42, 0xce, 0x36, 0x7c, 0x7a, 0x67, 0xf2, 0x83, 0x1d,
                            0xbf, 0x64, 0x08, 0x8d
                        ],
                        key_ring: None,
                        build_id: 53584,
                        versions_name: "10.2.5.53584".to_owned(),
                        product_config: [
                            0x53, 0x2, 0xd, 0x32, 0xe1, 0xa2, 0x56, 0x48, 0xc8, 0xe1, 0xea, 0xfd,
                            0x57, 0x71, 0x93, 0x5f
                        ]
                    },
                    Record {
                        region: "eu".to_owned(),
                        build_config: [
                            0x47, 0xe9, 0xe0, 0x6f, 0x83, 0x71, 0xaf, 0xb1, 0x41, 0xe2, 0x26, 0x14,
                            0xa9, 0x12, 0xac, 0xc8
                        ],
                        cdn_config: [
                            0x74, 0x09, 0x3d, 0x42, 0xce, 0x36, 0x7c, 0x7a, 0x67, 0xf2, 0x83, 0x1d,
                            0xbf, 0x64, 0x08, 0x8d
                        ],
                        key_ring: None,
                        build_id: 53584,
                        versions_name: "10.2.5.53584".to_owned(),
                        product_config: [
                            0x53, 0x2, 0xd, 0x32, 0xe1, 0xa2, 0x56, 0x48, 0xc8, 0xe1, 0xea, 0xfd,
                            0x57, 0x71, 0x93, 0x5f
                        ]
                    }
                ]
            }
        );
    }
}
