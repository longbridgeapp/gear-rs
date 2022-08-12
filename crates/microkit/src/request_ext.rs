use std::collections::HashMap;

use num_enum::FromPrimitive;
use poem_grpc::Request;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[repr(i64)]
pub enum BrokerType {
    #[num_enum(default)]
    Unknown,
    Trader,
    Broker,
}

macro_rules! define_values {
    ($($(#[$docs:meta])* ($method:ident, $ty:ty)),*) => {
        $(
            $(#[$docs])*
            #[allow(clippy::wrong_self_convention)]
            fn $method(&self) -> Option<$ty>;
        )*
    };
}

pub trait RequestExt {
    define_values!(
        (app_id, &str),
        (platform, &str),
        (member_id, u64),
        (accept_language, &str),
        (prefer_language, &str),
        (admin_id, u64),
        (cluster, &str),
        (from_cluster, &str),
        (base_level, i32),
        (ip_region, &str),
        (user_region, &str),
        (user_agent, &str),
        (application_version, &str),
        (application_build, &str),
        (bundle_id, &str),
        (device_id, &str),
        (device_name, &str),
        (device_model, &str),
        (op_member_id, u64),
        (organization_id, u64),
        (target_organization_id, u64),
        (target_aaid, u64),
        (email, &str),
        (account_channel, &str),
        (real_ip, &str)
    );

    fn market_levels(&self) -> HashMap<&str, Vec<&str>>;
    fn features(&self) -> Vec<&str>;
    fn broker_type(&self) -> Option<BrokerType>;
}

macro_rules! impl_string_values {
    ($($(#[$docs:meta])* ($method:ident, $name:literal)),*) => {
        $(
            $(#[$docs])*
            #[inline]
            fn $method(&self) -> Option<&str> {
                self.metadata().get($name)
            }
        )*
    };
}

macro_rules! impl_u64_values {
    ($($(#[$docs:meta])* ($method:ident, $name:literal)),*) => {
        $(
            $(#[$docs])*
            #[inline]
            fn $method(&self) -> Option<u64> {
                self.metadata().get($name).and_then(|value| value.parse().ok())
            }
        )*
    };
}

macro_rules! impl_i32_values {
    ($($(#[$docs:meta])* ($method:ident, $name:literal)),*) => {
        $(
            $(#[$docs])*
            #[inline]
            fn $method(&self) -> Option<i32> {
                self.metadata().get($name).and_then(|value| value.parse().ok())
            }
        )*
    };
}

impl<T> RequestExt for Request<T> {
    impl_string_values!(
        (app_id, "app-id"),
        (platform, "x-platform"),
        (accept_language, "accept-language"),
        (prefer_language, "x-prefer-language"),
        (cluster, "x-cluster"),
        (from_cluster, "x-from-cluster"),
        (ip_region, "ip-region"),
        (user_region, "user-region"),
        (user_agent, "x-user-agent"),
        (application_version, "x-application-version"),
        (application_build, "x-application-build"),
        (bundle_id, "x-bundle-id"),
        (device_id, "x-device-id"),
        (device_name, "x-device-name"),
        (device_model, "x-device-model"),
        (email, "x-email"),
        (account_channel, "account-channel"),
        (real_ip, "x-real-ip")
    );
    impl_u64_values!(
        (member_id, "member-id"),
        (admin_id, "admin-id"),
        (organization_id, "org-id"),
        (target_organization_id, "x-target-org-id"),
        (target_aaid, "target-aaid")
    );
    impl_i32_values!((base_level, "base-level"));

    fn market_levels(&self) -> HashMap<&str, Vec<&str>> {
        let mut levels = HashMap::new();

        if let Some(parts) = self.metadata().get("market-levels") {
            for kv in parts.split(';') {
                if let Some((key, values)) = kv.split_once(';') {
                    levels.insert(key, values.split(',').collect());
                }
            }
        }

        levels
    }

    fn features(&self) -> Vec<&str> {
        self.metadata()
            .get("x-features")
            .map(|value| value.split(',').collect())
            .unwrap_or_default()
    }

    fn op_member_id(&self) -> Option<u64> {
        self.metadata()
            .get("op-member-id")
            .and_then(|value| value.parse::<u64>().ok())
            .or_else(|| self.member_id())
    }

    fn broker_type(&self) -> Option<BrokerType> {
        self.metadata()
            .get("broker-type")
            .and_then(|value| value.parse::<i64>().ok())
            .map(Into::into)
    }
}
