//! Model for the summary response
use crate::response::base::{
    Dec4, Error, Record as BaseRecord, Response as BaseResponse, Result, String0,
};

/// A record in the summary response
#[derive(Debug, PartialEq)]
pub struct Record {
    /// The product name
    pub product: String0,

    /// The sequence number
    pub seqn: Dec4,

    /// The flags
    pub flags: String0,
}

impl<'input> TryFrom<BaseRecord<'_, 'input>> for Record {
    type Error = Error;

    fn try_from(record: BaseRecord<'_, 'input>) -> Result<Self> {
        let product = (&record
            .get_field_by_header_name("Product")
            .ok_or(Error::ExpectedField("Product"))?)
            .try_into()?;
        let seqn = (&record
            .get_field_by_header_name("Seqn")
            .ok_or(Error::ExpectedField("Seqn"))?)
            .try_into()?;
        let flags = (&record
            .get_field_by_header_name("Flags")
            .ok_or(Error::ExpectedField("Flags"))?)
            .try_into()?;

        Ok(Self {
            product,
            seqn,
            flags,
        })
    }
}

/// The summary response
#[derive(Debug, PartialEq)]
pub struct Response {
    /// The sequence number
    pub seqn: u32,

    /// The records
    pub records: Vec<Record>,
}

impl<'input> TryFrom<BaseResponse<'input>> for Response {
    type Error = Error;

    fn try_from(response: BaseResponse) -> Result<Self> {
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
        let input = "Product!STRING:0|Seqn!DEC:4|Flags!STRING:0\n\
        ## seqn = 2119172\n\
        agent|1476930|cdn\n\
        agent|2118018|\n\
        agent_beta|1476931|cdn\n\
        agent_beta|2110722|";

        let summary_response = Response::try_from(input).unwrap();

        assert_eq!(
            summary_response,
            Response {
                seqn: 2_119_172,
                records: vec![
                    Record {
                        product: "agent".to_owned(),
                        seqn: 1_476_930,
                        flags: "cdn".to_owned()
                    },
                    Record {
                        product: "agent".to_owned(),
                        seqn: 2_118_018,
                        flags: String::new()
                    },
                    Record {
                        product: "agent_beta".to_owned(),
                        seqn: 1_476_931,
                        flags: "cdn".to_owned()
                    },
                    Record {
                        product: "agent_beta".to_owned(),
                        seqn: 2_110_722,
                        flags: String::new()
                    }
                ]
            }
        );
    }
}
