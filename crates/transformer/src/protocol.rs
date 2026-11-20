// Generated from protocol/weft.protocol.json by protocol/generate.ts; do not edit.
//! Target names shared with the runtime.

/// Target platform; the string form matches `WEFT_PLATFORM`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Platform {
    Web,
    Wechat,
    Ios,
    Macos,
    Android,
    Windows,
    Linux,
}

impl Platform {
    /// Every platform, in declaration order.
    pub const ALL: [Platform; 7] = [
        Platform::Web,
        Platform::Wechat,
        Platform::Ios,
        Platform::Macos,
        Platform::Android,
        Platform::Windows,
        Platform::Linux,
    ];

    /// Name used by `WEFT_PLATFORM` and the Vite environments.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Platform::Web => "web",
            Platform::Wechat => "wechat",
            Platform::Ios => "ios",
            Platform::Macos => "macos",
            Platform::Android => "android",
            Platform::Windows => "windows",
            Platform::Linux => "linux",
        }
    }

    /// Whether the platform runs logic in the native kernel.
    #[must_use]
    pub fn is_native(self) -> bool {
        matches!(
            self,
            Self::Ios | Self::Macos | Self::Android | Self::Windows | Self::Linux
        )
    }

    /// Whether the platform uses the SwiftUI shell.
    #[must_use]
    pub fn is_apple(self) -> bool {
        matches!(self, Self::Ios | Self::Macos)
    }
}
