#[cfg(not(feature = "with_serde"))]
use alloc::vec::Vec;
#[cfg(not(feature = "with_serde"))]
use binary_sv2::{
    binary_codec_sv2, binary_codec_sv2::CVec, decodable::DecodableField, decodable::FieldMarker,
    free_vec, Error,
};
use binary_sv2::{Deserialize, GetSize, Serialize, Str0255};
use const_sv2::{
    SV2_JOB_DISTR_PROTOCOL_DISCRIMINANT, SV2_JOB_NEG_PROTOCOL_DISCRIMINANT,
    SV2_MINING_PROTOCOL_DISCRIMINANT, SV2_TEMPLATE_DISTR_PROTOCOL_DISCRIMINANT,
};
use core::convert::TryFrom;
#[cfg(not(feature = "with_serde"))]
use core::convert::TryInto;
#[cfg(feature = "with_serde")]
use serde_repr::*;

/// ## SetupConnection (Client -> Server)
/// Initiates the connection. This MUST be the first message sent by the client on the newly
/// opened connection. Server MUST respond with either a [`SetupConnectionSuccess`] or
/// [`SetupConnectionError`] message. Clients that are not configured to provide telemetry data to
/// the upstream node SHOULD set device_id to 0-length strings. However, they MUST always set
/// vendor to a string describing the manufacturer/developer and firmware version and SHOULD
/// always set hardware_version to a string describing, at least, the particular hardware/software
/// package in use.
///
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SetupConnection<'decoder> {
    /// [`Protocol`]
    pub protocol: Protocol,
    /// The minimum protocol version the client supports (currently must be 2).
    pub min_version: u16,
    /// The maximum protocol version the client supports (currently must be 2).
    pub max_version: u16,
    /// Flags indicating optional protocol features the client supports. Each
    /// protocol from [`SetupConnection.protocol`] field has its own values/flags.
    pub flags: u32,
    /// ASCII text indicating the hostname or IP address.
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub endpoint_host: Str0255<'decoder>,
    /// Connecting port value
    pub endpoint_port: u16,
    //-- DEVICE INFORMATION --//
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub vendor: Str0255<'decoder>,
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub hardware_version: Str0255<'decoder>,
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub firmware: Str0255<'decoder>,
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub device_id: Str0255<'decoder>,
}

impl<'decoder> SetupConnection<'decoder> {
    pub fn set_requires_standard_job(&mut self) {
        self.flags |= 0b_0000_0000_0000_0000_0000_0000_0000_0001
    }

    pub fn set_async_job_nogotiation(&mut self) {
        self.flags |= 0b_0000_0000_0000_0000_0000_0000_0000_0001
    }

    /// Check if passed flags support self flag
    pub fn check_flags(protocol: Protocol, available_flags: u32, required_flags: u32) -> bool {
        match protocol {
            // [0] [0] -> true
            // [0] [1] -> false
            // [1] [1] -> true
            // [0] [1] -> false
            Protocol::MiningProtocol => {
                let available = available_flags.reverse_bits();
                let required_flags = required_flags.reverse_bits();
                let requires_work_selection_passed = (required_flags >> 30) > 0;
                let requires_version_rolling_passed = (required_flags >> 29) > 0;

                let requires_work_selection_self = (available >> 30) > 0;
                let requires_version_rolling_self = (available >> 29) > 0;

                let work_selection =
                    !requires_work_selection_self || requires_work_selection_passed;
                let version_rolling =
                    !requires_version_rolling_self || requires_version_rolling_passed;

                work_selection && version_rolling
            }
            // TODO
            _ => todo!(),
        }
    }

    /// Check if passed versions support self versions if yes return the biggest version available
    pub fn get_version(&self, min_version: u16, max_version: u16) -> Option<u16> {
        if self.min_version > max_version || min_version > self.max_version {
            None
        } else {
            Some(self.max_version.min(max_version))
        }
    }

    pub fn requires_standard_job(&self) -> bool {
        has_requires_std_job(self.flags)
    }
}

pub fn has_requires_std_job(flags: u32) -> bool {
    let flags = flags.reverse_bits();
    let flag = flags >> 31;
    flag != 0
}
pub fn has_version_rolling(flags: u32) -> bool {
    let flags = flags.reverse_bits();
    let flags = flags << 1;
    let flag = flags >> 31;
    flag != 0
}
pub fn has_work_selection(flags: u32) -> bool {
    let flags = flags.reverse_bits();
    let flags = flags << 2;
    let flag = flags >> 31;
    flag != 0
}

#[repr(C)]
#[cfg(not(feature = "with_serde"))]
#[derive(Debug, Clone)]
pub struct CSetupConnection {
    pub protocol: Protocol,
    pub min_version: u16,
    pub max_version: u16,
    pub flags: u32,
    pub endpoint_host: CVec,
    pub endpoint_port: u16,
    pub vendor: CVec,
    pub hardware_version: CVec,
    pub firmware: CVec,
    pub device_id: CVec,
}

#[cfg(not(feature = "with_serde"))]
impl<'a> CSetupConnection {
    #[cfg(not(feature = "with_serde"))]
    #[allow(clippy::wrong_self_convention)]
    pub fn to_rust_rep_mut(&'a mut self) -> Result<SetupConnection<'a>, Error> {
        let endpoint_host: Str0255 = self.endpoint_host.as_mut_slice().try_into()?;
        let vendor: Str0255 = self.vendor.as_mut_slice().try_into()?;
        let hardware_version: Str0255 = self.hardware_version.as_mut_slice().try_into()?;
        let firmware: Str0255 = self.firmware.as_mut_slice().try_into()?;
        let device_id: Str0255 = self.device_id.as_mut_slice().try_into()?;

        Ok(SetupConnection {
            protocol: self.protocol,
            min_version: self.min_version,
            max_version: self.max_version,
            flags: self.flags,
            endpoint_host,
            endpoint_port: self.endpoint_port,
            vendor,
            hardware_version,
            firmware,
            device_id,
        })
    }
}

#[no_mangle]
#[cfg(not(feature = "with_serde"))]
pub extern "C" fn free_setup_connection(s: CSetupConnection) {
    drop(s)
}

#[cfg(not(feature = "with_serde"))]
impl Drop for CSetupConnection {
    fn drop(&mut self) {
        free_vec(&mut self.endpoint_host);
        free_vec(&mut self.vendor);
        free_vec(&mut self.hardware_version);
        free_vec(&mut self.firmware);
        free_vec(&mut self.device_id);
    }
}

#[cfg(not(feature = "with_serde"))]
impl<'a> From<SetupConnection<'a>> for CSetupConnection {
    fn from(v: SetupConnection) -> Self {
        Self {
            protocol: v.protocol,
            min_version: v.min_version,
            max_version: v.max_version,
            flags: v.flags,
            endpoint_host: v.endpoint_host.into(),
            endpoint_port: v.endpoint_port,
            vendor: v.vendor.into(),
            hardware_version: v.hardware_version.into(),
            firmware: v.firmware.into(),
            device_id: v.device_id.into(),
        }
    }
}

/// ## SetupConnection.Success (Server -> Client)
/// Response to [`SetupConnection`] message if the server accepts the connection. The client is
/// required to verify the set of feature flags that the server supports and act accordingly.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy)]
#[repr(C)]
pub struct SetupConnectionSuccess {
    /// Selected version proposed by the connecting node that the upstream
    /// node supports. This version will be used on the connection for the rest
    /// of its life.
    pub used_version: u16,
    /// Flags indicating optional protocol features the server supports. Each
    /// protocol from [`Protocol`] field has its own values/flags.
    pub flags: u32,
}

/// ## SetupConnection.Error (Server -> Client)
/// When protocol version negotiation fails (or there is another reason why the upstream node
/// cannot setup the connection) the server sends this message with a particular error code prior
/// to closing the connection.
/// In order to allow a client to determine the set of available features for a given server (e.g. for
/// proxies which dynamically switch between different pools and need to be aware of supported
/// options), clients SHOULD send a SetupConnection message with all flags set and examine the
/// (potentially) resulting [`SetupConnectionError`] message’s flags field. The Server MUST provide
/// the full set of flags which it does not support in each [`SetupConnectionError`] message and
/// MUST consistently support the same set of flags across all servers on the same hostname and
/// port number. If flags is 0, the error is a result of some condition aside from unsupported flags.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SetupConnectionError<'decoder> {
    /// Flags indicating features causing an error.
    pub flags: u32,
    /// Human-readable error code(s). See Error Codes section, [link].
    /// ### Possible error codes:
    /// * ‘unsupported-feature-flags’
    /// * ‘unsupported-protocol’
    /// * ‘protocol-version-mismatch’
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub error_code: Str0255<'decoder>,
}

#[repr(C)]
#[cfg(not(feature = "with_serde"))]
#[derive(Debug, Clone)]
pub struct CSetupConnectionError {
    flags: u32,
    error_code: CVec,
}

#[cfg(not(feature = "with_serde"))]
impl<'a> CSetupConnectionError {
    #[cfg(not(feature = "with_serde"))]
    #[allow(clippy::wrong_self_convention)]
    pub fn to_rust_rep_mut(&'a mut self) -> Result<SetupConnectionError<'a>, Error> {
        let error_code: Str0255 = self.error_code.as_mut_slice().try_into()?;

        Ok(SetupConnectionError {
            flags: self.flags,
            error_code,
        })
    }
}

#[no_mangle]
#[cfg(not(feature = "with_serde"))]
pub extern "C" fn free_setup_connection_error(s: CSetupConnectionError) {
    drop(s)
}

#[cfg(not(feature = "with_serde"))]
impl Drop for CSetupConnectionError {
    fn drop(&mut self) {
        free_vec(&mut self.error_code);
    }
}

#[cfg(not(feature = "with_serde"))]
impl<'a> From<SetupConnectionError<'a>> for CSetupConnectionError {
    fn from(v: SetupConnectionError<'a>) -> Self {
        Self {
            flags: v.flags,
            error_code: v.error_code.into(),
        }
    }
}

/// MiningProtocol = [`SV2_MINING_PROTOCOL_DISCRIMINANT`],
/// JobDeclarationProtocol = [`SV2_JOB_NEG_PROTOCOL_DISCRIMINANT`],
/// TemplateDistributionProtocol = [`SV2_TEMPLATE_DISTR_PROTOCOL_DISCRIMINANT`],
/// JobDistributionProtocol = [`SV2_JOB_DISTR_PROTOCOL_DISCRIMINANT`],
#[cfg_attr(feature = "with_serde", derive(Serialize_repr, Deserialize_repr))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(clippy::enum_variant_names)]
pub enum Protocol {
    MiningProtocol = SV2_MINING_PROTOCOL_DISCRIMINANT,
    JobDeclarationProtocol = SV2_JOB_NEG_PROTOCOL_DISCRIMINANT,
    TemplateDistributionProtocol = SV2_TEMPLATE_DISTR_PROTOCOL_DISCRIMINANT,
    JobDistributionProtocol = SV2_JOB_DISTR_PROTOCOL_DISCRIMINANT,
}

#[cfg(not(feature = "with_serde"))]
impl<'a> From<Protocol> for binary_sv2::encodable::EncodableField<'a> {
    fn from(v: Protocol) -> Self {
        let val = v as u8;
        val.into()
    }
}

#[cfg(not(feature = "with_serde"))]
impl<'decoder> binary_sv2::Decodable<'decoder> for Protocol {
    fn get_structure(
        _: &[u8],
    ) -> core::result::Result<alloc::vec::Vec<FieldMarker>, binary_sv2::Error> {
        let field: FieldMarker = 0_u8.into();
        Ok(alloc::vec![field])
    }
    fn from_decoded_fields(
        mut v: alloc::vec::Vec<DecodableField<'decoder>>,
    ) -> core::result::Result<Self, binary_sv2::Error> {
        let val = v.pop().ok_or(binary_sv2::Error::NoDecodableFieldPassed)?;
        let val: u8 = val.try_into()?;
        val.try_into()
            .map_err(|_| binary_sv2::Error::ValueIsNotAValidProtocol(val))
    }
}

impl TryFrom<u8> for Protocol {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            SV2_MINING_PROTOCOL_DISCRIMINANT => Ok(Protocol::MiningProtocol),
            SV2_JOB_NEG_PROTOCOL_DISCRIMINANT => Ok(Protocol::JobDeclarationProtocol),
            SV2_TEMPLATE_DISTR_PROTOCOL_DISCRIMINANT => Ok(Protocol::TemplateDistributionProtocol),
            SV2_JOB_DISTR_PROTOCOL_DISCRIMINANT => Ok(Protocol::JobDistributionProtocol),
            _ => Err(()),
        }
    }
}

impl GetSize for Protocol {
    fn get_size(&self) -> usize {
        1
    }
}

#[cfg(feature = "with_serde")]
impl From<Protocol> for u8 {
    fn from(val: Protocol) -> Self {
        match val {
            Protocol::MiningProtocol => SV2_MINING_PROTOCOL_DISCRIMINANT,
            Protocol::JobDeclarationProtocol => SV2_JOB_NEG_PROTOCOL_DISCRIMINANT,
            Protocol::TemplateDistributionProtocol => SV2_TEMPLATE_DISTR_PROTOCOL_DISCRIMINANT,
            Protocol::JobDistributionProtocol => SV2_JOB_DISTR_PROTOCOL_DISCRIMINANT,
        }
    }
}

#[cfg(feature = "with_serde")]
impl<'d> GetSize for SetupConnectionError<'d> {
    fn get_size(&self) -> usize {
        self.flags.get_size() + self.error_code.get_size()
    }
}
#[cfg(feature = "with_serde")]
impl GetSize for SetupConnectionSuccess {
    fn get_size(&self) -> usize {
        self.used_version.get_size() + self.flags.get_size()
    }
}
#[cfg(feature = "with_serde")]
impl<'d> GetSize for SetupConnection<'d> {
    fn get_size(&self) -> usize {
        self.protocol.get_size()
            + self.min_version.get_size()
            + self.max_version.get_size()
            + self.flags.get_size()
            + self.endpoint_host.get_size()
            + self.endpoint_port.get_size()
            + self.vendor.get_size()
            + self.hardware_version.get_size()
            + self.firmware.get_size()
            + self.device_id.get_size()
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::alloc::string::ToString;
    use core::convert::TryInto;

    #[test]
    fn test_check_flag() {
        let protocol = crate::Protocol::MiningProtocol;
        let flag_available = 0b_0000_0000_0000_0000_0000_0000_0000_0000;
        let flag_required = 0b_0000_0000_0000_0000_0000_0000_0000_0001;
        assert!(SetupConnection::check_flags(
            protocol,
            flag_available,
            flag_required
        ));
    }

    #[test]
    fn test_has_requires_std_job() {
        let flags = 0b_0000_0000_0000_0000_0000_0000_0000_0001;
        assert_eq!(has_requires_std_job(flags), true);
        let flags = 0b_0000_0000_0000_0000_0000_0000_0000_0010;
        assert_eq!(has_requires_std_job(flags), false);
    }

    #[test]
    fn test_has_version_rolling() {
        let flags = 0b_0000_0000_0000_0000_0000_0000_0000_0010;
        assert_eq!(has_version_rolling(flags), true);
        let flags = 0b_0000_0000_0000_0000_0000_0000_0000_0001;
        assert_eq!(has_version_rolling(flags), false);
    }

    #[test]
    fn test_has_work_selection() {
        let flags = 0b_0000_0000_0000_0000_0000_0000_0000_0100;
        assert_eq!(has_work_selection(flags), true);
        let flags = 0b_0000_0000_0000_0000_0000_0000_0000_0001;
        assert_eq!(has_work_selection(flags), false);
    }

    fn create_setup_connection() -> SetupConnection<'static> {
        SetupConnection {
            protocol: Protocol::MiningProtocol,
            min_version: 1,
            max_version: 4,
            flags: 0,
            endpoint_host: "0.0.0.0".to_string().into_bytes().try_into().unwrap(),
            endpoint_port: 0,
            vendor: "vendor".to_string().into_bytes().try_into().unwrap(),
            hardware_version: "hw_version".to_string().into_bytes().try_into().unwrap(),
            firmware: "firmware".to_string().into_bytes().try_into().unwrap(),
            device_id: "device_id".to_string().into_bytes().try_into().unwrap(),
        }
    }

    #[test]
    fn test_get_version() {
        let setup_conn = create_setup_connection();
        assert_eq!(setup_conn.get_version(1, 5).unwrap(), 4);
        assert_eq!(setup_conn.get_version(6, 6), None);
    }

    // Test SetupConnection::set_requires_std_job
    #[test]
    fn test_set_requires_std_job() {
        let mut setup_conn = create_setup_connection();
        assert!(!setup_conn.requires_standard_job());
        setup_conn.set_requires_standard_job();
        assert!(setup_conn.requires_standard_job());
    }
}
