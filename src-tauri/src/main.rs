// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Work around WebKitGTK's DMA-BUF renderer on Wayland.
///
/// WebKitGTK negotiates buffer formats that some Wayland compositors and GPU
/// drivers reject. On KDE Plasma with the NVIDIA driver this shows up as a
/// blank/white window, flickering while resizing, or a hard crash with
/// `Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display.`
/// See https://github.com/tauri-apps/tauri/issues/9394
///
/// These variables only take effect if they are set before the webview (and
/// thus GTK/WebKit) is initialized, so this must run before `run()`. A value
/// already present in the environment is never overridden, so users who want
/// the faster DMA-BUF path can still opt back in themselves.
#[cfg(target_os = "linux")]
fn apply_linux_wayland_graphics_workarounds() {
    use std::env;

    let on_wayland = env::var_os("WAYLAND_DISPLAY").is_some()
        || env::var("XDG_SESSION_TYPE")
            .map(|session| session.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false);

    if !on_wayland {
        return;
    }

    // Disables the accelerated DMA-BUF path that fails to build a framebuffer
    // on the affected compositor/driver combinations.
    if env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    // The NVIDIA proprietary driver's explicit-sync path is the other half of
    // the Wayland "Error 71" protocol error. Checking for the loaded kernel
    // module keeps this off non-NVIDIA systems.
    let on_nvidia = std::path::Path::new("/sys/module/nvidia").exists();
    if on_nvidia && env::var_os("__NV_DISABLE_EXPLICIT_SYNC").is_none() {
        env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    apply_linux_wayland_graphics_workarounds();

    tauri_app_lib::run()
}
