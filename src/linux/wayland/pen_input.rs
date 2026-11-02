// Added by ibis inc. (not part of upstream gpui-pre-linux; Apache-2.0).
//
// Pure helpers for turning Wayland tablet (zwp_tablet_v2) and wl_touch events
// into mouse input. They do not depend on GPUI or wayland-client types so that
// they can be unit tested without a compositor.

/// The mouse button a pen / touch event is turned into.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PenButton {
    Left,
    Right,
    Middle,
}

// Values from <linux/input-event-codes.h>.
const BTN_LEFT: u32 = 0x110;
const BTN_RIGHT: u32 = 0x111;
const BTN_MIDDLE: u32 = 0x112;
const BTN_STYLUS3: u32 = 0x149;
const BTN_STYLUS: u32 = 0x14b;
const BTN_STYLUS2: u32 = 0x14c;

/// The button produced while the pen tip touches the tablet (tablet_tool.down / up).
/// Every tool type (pen, eraser, brush, ...) uses the left button.
pub(crate) fn tip_button() -> PenButton {
    PenButton::Left
}

/// Maps the button code of tablet_tool.button to a mouse button.
/// The first barrel button is the secondary (right) button, the second is the
/// middle button, as for the usual pen drivers. Other codes are ignored.
pub(crate) fn stylus_button(code: u32) -> Option<PenButton> {
    match code {
        BTN_STYLUS => Some(PenButton::Right),
        BTN_STYLUS2 => Some(PenButton::Middle),
        // The third barrel button has no mouse equivalent.
        BTN_STYLUS3 => None,
        // A mouse tool on a tablet sends the regular mouse buttons.
        BTN_LEFT => Some(PenButton::Left),
        BTN_RIGHT => Some(PenButton::Right),
        BTN_MIDDLE => Some(PenButton::Middle),
        _ => None,
    }
}

/// Follows the touch points of wl_touch. Only the first finger is turned into
/// mouse input; other fingers are ignored until it is lifted.
#[derive(Default, Debug)]
pub(crate) struct TouchTracker {
    primary: Option<i32>,
}

impl TouchTracker {
    /// A finger touched down. Returns true if it is the primary finger.
    pub(crate) fn down(&mut self, id: i32) -> bool {
        if self.primary.is_none() {
            self.primary = Some(id);
            true
        } else {
            false
        }
    }

    pub(crate) fn is_primary(&self, id: i32) -> bool {
        self.primary == Some(id)
    }

    /// A finger was lifted. Returns true if it was the primary finger.
    pub(crate) fn up(&mut self, id: i32) -> bool {
        if self.is_primary(id) {
            self.primary = None;
            true
        } else {
            false
        }
    }

    pub(crate) fn clear(&mut self) {
        self.primary = None;
    }

    pub(crate) fn is_active(&self) -> bool {
        self.primary.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tip_is_left() {
        assert_eq!(tip_button(), PenButton::Left);
    }

    #[test]
    fn barrel_buttons() {
        assert_eq!(stylus_button(0x14b), Some(PenButton::Right));
        assert_eq!(stylus_button(0x14c), Some(PenButton::Middle));
        assert_eq!(stylus_button(0x149), None);
        assert_eq!(stylus_button(0x110), Some(PenButton::Left));
        assert_eq!(stylus_button(0x14a), None); // BTN_TOUCH
        assert_eq!(stylus_button(0), None);
    }

    #[test]
    fn only_first_finger_is_primary() {
        let mut t = TouchTracker::default();
        assert!(!t.is_active());
        assert!(t.down(3));
        assert!(!t.down(4));
        assert!(t.is_primary(3));
        assert!(!t.is_primary(4));
        assert!(!t.up(4));
        assert!(t.is_active());
        assert!(t.up(3));
        assert!(!t.is_active());
        assert!(t.down(4));
        t.clear();
        assert!(!t.is_active());
    }
}
