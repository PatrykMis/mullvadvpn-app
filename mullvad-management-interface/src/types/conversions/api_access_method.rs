/// Implements conversions for the auxilliary
/// [`crate::types::proto::ApiAccessMethodSettings`] type to the internal
/// [`mullvad_types::access_method::Settings`] data type.
mod settings {
    use crate::types::{proto, FromProtobufTypeError};
    use mullvad_types::access_method;

    impl From<&api_access::Settings> for proto::ApiAccessMethodSettings {
        fn from(settings: &api_access::Settings) -> Self {
            Self {
                api_access_methods: settings
                    .api_access_methods
                    .iter()
                    .map(|method| method.clone().into())
                    .collect(),
            }
        }
    }

    impl From<api_access::Settings> for proto::ApiAccessMethodSettings {
        fn from(settings: api_access::Settings) -> Self {
            proto::ApiAccessMethodSettings::from(&settings)
        }
    }

    impl TryFrom<proto::ApiAccessMethodSettings> for api_access::Settings {
        type Error = FromProtobufTypeError;

        fn try_from(settings: proto::ApiAccessMethodSettings) -> Result<Self, Self::Error> {
            Ok(Self {
                api_access_methods: settings
                    .api_access_methods
                    .iter()
                    .map(api_access::ApiAccessMethod::try_from)
                    .collect::<Result<Vec<api_access::ApiAccessMethod>, _>>()?,
            })
        }
    }

    impl From<access_method::daemon::ApiAccessMethodUpdate> for proto::ApiAccessMethodUpdate {
        fn from(value: access_method::daemon::ApiAccessMethodUpdate) -> Self {
            proto::ApiAccessMethodUpdate {
                id: Some(proto::Uuid::from(value.id)),
                access_method: Some(proto::ApiAccessMethod::from(value.access_method)),
            }
        }
    }

    impl TryFrom<proto::ApiAccessMethodUpdate> for access_method::daemon::ApiAccessMethodUpdate {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::ApiAccessMethodUpdate) -> Result<Self, Self::Error> {
            let api_access_method = value
                .access_method
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not convert Access Method from protobuf",
                ))
                .and_then(api_access::ApiAccessMethod::try_from)?;

            let id = value
                .id
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not convert Access Method from protobuf",
                ))
                .map(api_access::ApiAccessMethodId::from)?;

            Ok(access_method::daemon::ApiAccessMethodUpdate {
                id,
                access_method: api_access_method,
            })
        }
    }
}

/// Implements conversions for the auxilliary
/// [`crate::types::proto::ApiAccessMethod`] type to the internal
/// [`mullvad_types::access_method::AccessMethod`] data type.
mod data {
    use crate::types::{proto, FromProtobufTypeError};
    use mullvad_types::api_access::{
        AccessMethod, ApiAccessMethod, ApiAccessMethodId, BuiltInAccessMethod, CustomAccessMethod,
        Shadowsocks, Socks5, Socks5Local, Socks5Remote,
    };

    impl TryFrom<proto::ApiAccessMethods> for Vec<ApiAccessMethod> {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::ApiAccessMethods) -> Result<Self, Self::Error> {
            value
                .api_access_methods
                .iter()
                .map(ApiAccessMethod::try_from)
                .collect()
        }
    }

    impl TryFrom<proto::ApiAccessMethod> for ApiAccessMethod {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::ApiAccessMethod) -> Result<Self, Self::Error> {
            let id = value
                .id
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not deserialize Access Method from protobuf",
                ))
                .and_then(ApiAccessMethodId::try_from)?;
            let name = value.name;
            let enabled = value.enabled;
            let access_method = value
                .access_method
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not deserialize Access Method from protobuf",
                ))
                .and_then(AccessMethod::try_from)?;

            Ok(AccessMethodSetting::with_id(
                id,
                name,
                enabled,
                access_method,
            ))
        }
    }

    impl From<AccessMethodSetting> for proto::ApiAccessMethod {
        fn from(value: AccessMethodSetting) -> Self {
            let id = proto::Uuid::from(value.get_id());
            let name = value.get_name();
            let enabled = value.enabled();
            proto::ApiAccessMethod {
                id: Some(id),
                name,
                enabled,
                access_method: Some(proto::AccessMethod::from(value.access_method)),
            }
        }
    }

    impl TryFrom<proto::AccessMethod> for AccessMethod {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::AccessMethod) -> Result<Self, Self::Error> {
            let access_method =
                value
                    .access_method
                    .ok_or(FromProtobufTypeError::InvalidArgument(
                        "Could not deserialize Access Method from protobuf",
                    ))?;

            let access_method = match access_method {
                proto::api_access_method::AccessMethod::Direct(
                    proto::api_access_method::Direct {},
                ) => AccessMethod::from(BuiltInAccessMethod::Direct),

                proto::api_access_method::AccessMethod::Bridges(
                    proto::api_access_method::Bridges {},
                ) => AccessMethod::from(BuiltInAccessMethod::Bridge),
                proto::api_access_method::AccessMethod::Socks5local(local) => {
                    let socks = Socks5Local::from_args(
                        local.ip,
                        local.port as u16,
                        local.local_port as u16,
                    )
                    .ok_or(FromProtobufTypeError::InvalidArgument(
                        "Could not parse Socks5 (local) message from protobuf",
                    ))?;
                    AccessMethod::from(socks)
                }

                proto::api_access_method::AccessMethod::Socks5remote(remote) => {
                    let socks = Socks5Remote::from_args(remote.ip, remote.port as u16).ok_or({
                        FromProtobufTypeError::InvalidArgument(
                            "Could not parse Socks5 (remote) message from protobuf",
                        )
                    })?;
                    AccessMethod::from(socks)
                }
                proto::api_access_method::AccessMethod::Shadowsocks(ss) => {
                    let socks =
                        Shadowsocks::from_args(ss.ip, ss.port as u16, ss.cipher, ss.password)
                            .ok_or(FromProtobufTypeError::InvalidArgument(
                                "Could not parse Shadowsocks message from protobuf",
                            ))?;
                    AccessMethod::from(socks)
                }
            };

            Ok(ApiAccessMethod::with_id(id, name, enabled, access_method))
        }
    }

    impl From<ApiAccessMethodId> for proto::Uuid {
        fn from(value: ApiAccessMethodId) -> Self {
            proto::Uuid {
                value: value.to_string(),
            }
        }
    }

    impl TryFrom<proto::Uuid> for ApiAccessMethodId {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::Uuid) -> Result<Self, Self::Error> {
            Self::from_string(value.value).ok_or(FromProtobufTypeError::InvalidArgument(
                "Could not parse UUID message from protobuf",
            ))
        }
    }

    impl From<ApiAccessMethod> for proto::ApiAccessMethod {
        fn from(value: ApiAccessMethod) -> Self {
            let id = proto::Uuid::from(value.get_id());
            let name = value.get_name();
            let enabled = value.enabled();
            let access_method = match value.access_method {
                AccessMethod::Custom(value) => match value {
                    CustomAccessMethod::Shadowsocks(ss) => {
                        proto::api_access_method::AccessMethod::Shadowsocks(
                            proto::api_access_method::Shadowsocks {
                                ip: ss.peer.ip().to_string(),
                                port: ss.peer.port() as u32,
                                password: ss.password,
                                cipher: ss.cipher,
                            },
                        )
                    }
                    CustomAccessMethod::Socks5(Socks5::Local(Socks5Local { peer, port })) => {
                        proto::api_access_method::AccessMethod::Socks5local(
                            proto::api_access_method::Socks5Local {
                                ip: peer.ip().to_string(),
                                port: peer.port() as u32,
                                local_port: port as u32,
                            },
                        )
                    }
                    CustomAccessMethod::Socks5(Socks5::Remote(Socks5Remote { peer })) => {
                        proto::api_access_method::AccessMethod::Socks5remote(
                            proto::api_access_method::Socks5Remote {
                                ip: peer.ip().to_string(),
                                port: peer.port() as u32,
                            },
                        )
                    }
                },
                AccessMethod::BuiltIn(value) => match value {
                    mullvad_types::api_access::BuiltInAccessMethod::Direct => {
                        proto::api_access_method::AccessMethod::Direct(
                            proto::api_access_method::Direct {},
                        )
                    }
                    mullvad_types::api_access::BuiltInAccessMethod::Bridge => {
                        proto::api_access_method::AccessMethod::Bridges(
                            proto::api_access_method::Bridges {},
                        )
                    }
                },
            };

            proto::ApiAccessMethod {
                id: Some(id),
                name,
                enabled,
                access_method: Some(access_method),
            }
        }
    }

    impl TryFrom<&proto::ApiAccessMethod> for ApiAccessMethod {
        type Error = FromProtobufTypeError;

        fn try_from(value: &proto::ApiAccessMethod) -> Result<Self, Self::Error> {
            ApiAccessMethod::try_from(value.clone())
        }
    }

    impl From<Vec<ApiAccessMethod>> for proto::ApiAccessMethods {
        fn from(value: Vec<ApiAccessMethod>) -> proto::ApiAccessMethods {
            proto::ApiAccessMethods {
                api_access_methods: value.iter().map(|method| method.clone().into()).collect(),
            }
        }
    }
}
