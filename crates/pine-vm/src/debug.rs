use std::sync::OnceLock;

pub(crate) fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("PINE_VM_DEBUG")
            .map(|value| {
                let normalized = value.to_string_lossy().trim().to_ascii_lowercase();
                !matches!(normalized.as_str(), "" | "0" | "false" | "off" | "no")
            })
            .unwrap_or(false)
    })
}

macro_rules! vm_debug {
    ($($arg:tt)*) => {
        if $crate::debug::enabled() {
            eprintln!($($arg)*);
        }
    };
}

pub(crate) use vm_debug;
