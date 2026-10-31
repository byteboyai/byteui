//! Serialize global-theme tests and restore the exact entry state on unwind.
use super::{color, font, geometry, icon_size};
use std::sync::{Mutex, MutexGuard};
static GLOBAL_STATE_LOCK: Mutex<()> = Mutex::new(());

pub(crate) struct GlobalStateGuard {
    _lock: MutexGuard<'static, ()>,
    colors: color::ThemeSnapshot,
    icons: Option<icon_size::TestSnapshot>,
    fonts: font::FontTokens,
    geometry: geometry::GeometryTokens,
}
impl Drop for GlobalStateGuard {
    fn drop(&mut self) {
        color::restore_test_snapshot(self.colors);
        font::set_theme(self.fonts);
        geometry::set_theme(self.geometry);
        icon_size::restore_test_snapshot(self.icons.take().expect("snapshot restored once"));
    }
}
pub(crate) fn lock_global_state() -> GlobalStateGuard {
    let lock = GLOBAL_STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    GlobalStateGuard {
        _lock: lock,
        colors: color::current_snapshot(),
        icons: Some(icon_size::test_snapshot()),
        fonts: font::current(),
        geometry: geometry::current(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guard_restores_state_after_unwind() {
        // Hold the serialization lock throughout the probe to exclude other tests.
        let outer = lock_global_state();
        let colors = color::current_snapshot();
        let scale = icon_size::scale();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _restore = GlobalStateGuard {
                // A separate local mutex avoids recursively locking the global lock.
                _lock: LOCAL.lock().unwrap_or_else(|e| e.into_inner()),
                colors,
                icons: Some(icon_size::test_snapshot()),
                fonts: font::current(),
                geometry: geometry::current(),
            };
            color::set_scheme(color::ColorScheme::Light);
            icon_size::set_scale(2.75);
            panic!("intentional unwind");
        }));
        assert!(result.is_err());
        assert_eq!(color::current_snapshot(), colors);
        assert_eq!(icon_size::scale(), scale);
        drop(outer);
    }
    static LOCAL: Mutex<()> = Mutex::new(());
}
