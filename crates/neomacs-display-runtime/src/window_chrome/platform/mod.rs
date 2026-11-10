use neomacs_display_protocol::{Color, NativeChromeSupport, UnsupportedChrome, WindowChromePolicy};
use winit::window::{Window, WindowAttributes};

#[cfg(target_os = "macos")]
mod macos;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ChromeError {
    #[error("unsupported native chrome: {0:?}")]
    Unsupported(UnsupportedChrome),
    #[cfg(target_os = "macos")]
    #[error("AppKit window is unavailable or called outside the main thread")]
    Unavailable,
}

pub(super) fn prepare(attrs: WindowAttributes, policy: WindowChromePolicy) -> WindowAttributes {
    std::cfg_select! {
        target_os = "macos" => { macos::prepare(attrs, policy) }
        _ => { let _ = policy; attrs }
    }
}

pub(super) fn apply(
    window: &dyn Window,
    policy: WindowChromePolicy,
    color: Color,
    appearance: crate::thread_comm::WindowAppearance,
) -> Result<(), ChromeError> {
    let support = std::cfg_select! {
        target_os = "macos" => { NativeChromeSupport::AppKit }
        _ => { NativeChromeSupport::SystemOnly }
    };
    policy.resolve(support).map_err(ChromeError::Unsupported)?;
    // GNU's `ns-appearance' picks the window's NSAppearance
    // (`[EmacsWindow setAppearance]', src/nsterm.m:10358-10381); winit maps a
    // theme onto the same property, and `None' onto following the system.
    // Other platforms take it as their own light/dark title bar hint.
    window.set_theme(match appearance {
        crate::thread_comm::WindowAppearance::Dark => Some(winit::window::Theme::Dark),
        crate::thread_comm::WindowAppearance::Light => Some(winit::window::Theme::Light),
        crate::thread_comm::WindowAppearance::System => None,
    });
    std::cfg_select! {
        target_os = "macos" => { macos::apply(window, policy, color) }
        _ => { let _ = (window, color); Ok(()) }
    }
}
