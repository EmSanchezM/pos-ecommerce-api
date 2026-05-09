//! Concrete adapter implementations that wire `modules/*` outbound traits to
//! cross-module use cases. Lives in api-gateway because that's the only
//! crate that depends on every module simultaneously.

pub mod subscription_billing_stubs;
