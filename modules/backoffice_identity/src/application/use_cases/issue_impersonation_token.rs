// Impersonation token policy constant.
//
// The v1 local-signing use case (`IssueImpersonationTokenUseCase`) was removed
// in impersonation v2: the backoffice no longer signs tenant tokens. Minting now
// happens in api-gateway's internal endpoint, reached via the
// `ImpersonationTokenIssuer` port. Only the policy constant lives on here, since
// it is shared by the audited flow and the response DTO.

/// Constant expiry for impersonation tokens — 15 minutes (NFR-SEC-4).
///
/// Any code path that issues with a longer duration is a security defect.
pub const IMPERSONATION_TOKEN_EXPIRY_SECONDS: i64 = 900;
