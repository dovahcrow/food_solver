//! Build provenance for the binary that embeds this crate.
//!
//! `build.rs` stamps two values at compile time, so a running binary can say
//! which revision it came from: the moment the crate was built (ISO 8601 UTC)
//! and the git revision of the checkout it was built from.

use std::fmt;

use serde::Serialize;

/// When this crate was built, as an ISO 8601 UTC timestamp.
pub const BUILD_DATE: &str = env!("FOOD_BUILD_DATE");

/// The git revision this build came from, or `unknown` outside a checkout.
pub const GIT_SHA: &str = env!("FOOD_GIT_SHA");

/// Build provenance: when the crate was built and from which revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BuildInfo {
    /// ISO 8601 UTC, e.g. `2026-09-29T08:15:00Z`.
    pub build_date: &'static str,
    /// Short git revision, or `unknown` when the build had no checkout.
    pub git_sha: &'static str,
}

/// The provenance of this build.
pub const BUILD_INFO: BuildInfo = BuildInfo {
    build_date: BUILD_DATE,
    git_sha: GIT_SHA,
};

impl fmt::Display for BuildInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "built at {} (git {})",
            self.build_date, self.git_sha
        )
    }
}
