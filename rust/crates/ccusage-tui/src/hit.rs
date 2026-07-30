//! Per-frame map from screen regions to actions. Rebuilt on every draw so
//! mouse clicks dispatch the same actions as the keyboard.
use ratatui::layout::Rect;

use crate::action::Action;

#[derive(Default)]
pub(crate) struct HitMap {
    targets: Vec<(Rect, Action)>,
    popup: Option<Rect>,
    popup_targets: Vec<(Rect, Action)>,
    scrollbar: Option<(Rect, usize)>,
}

impl HitMap {
    pub(crate) fn clear(&mut self) {
        self.targets.clear();
        self.popup = None;
        self.popup_targets.clear();
        self.scrollbar = None;
    }

    /// Marks the vertical scrollbar track and the row count it spans, so
    /// clicks and drags on it can jump proportionally.
    pub(crate) fn set_scrollbar(&mut self, track: Rect, row_count: usize) {
        self.scrollbar = Some((track, row_count));
    }

    /// A click or drag on the scrollbar track, mapped to the row at that
    /// fraction of the table.
    pub(crate) fn scrollbar_jump(&self, x: u16, y: u16) -> Option<Action> {
        let (track, row_count) = self.scrollbar?;
        if !contains(track, x, y) || track.height == 0 || row_count == 0 {
            return None;
        }
        let fraction = f64::from(y - track.y) / f64::from(track.height.saturating_sub(1).max(1));
        let index = (fraction * (row_count - 1) as f64).round() as usize;
        Some(Action::SelectRow(index.min(row_count - 1)))
    }

    pub(crate) fn register(&mut self, rect: Rect, action: Action) {
        self.targets.push((rect, action));
    }

    /// Marks the modal popup region; while set, clicks outside it close the
    /// popup instead of hitting the targets underneath.
    pub(crate) fn set_popup(&mut self, rect: Rect) {
        self.popup = Some(rect);
    }

    /// Registers a clickable region inside the popup, which shadows the
    /// regular targets while the popup is open.
    pub(crate) fn register_popup(&mut self, rect: Rect, action: Action) {
        self.popup_targets.push((rect, action));
    }

    pub(crate) fn popup_hit(&self, x: u16, y: u16) -> Option<Action> {
        self.popup_targets
            .iter()
            .find(|(rect, _)| contains(*rect, x, y))
            .map(|(_, action)| *action)
    }

    pub(crate) fn popup_contains(&self, x: u16, y: u16) -> bool {
        self.popup.is_some_and(|rect| contains(rect, x, y))
    }

    pub(crate) fn popup_active(&self) -> bool {
        self.popup.is_some()
    }

    pub(crate) fn hit(&self, x: u16, y: u16) -> Option<Action> {
        self.targets
            .iter()
            .find(|(rect, _)| contains(*rect, x, y))
            .map(|(_, action)| *action)
    }

    /// The table row under the cursor, if any, for hover highlighting.
    pub(crate) fn row_at(&self, x: u16, y: u16) -> Option<usize> {
        match self.hit(x, y) {
            Some(Action::ClickRow(index)) => Some(index),
            _ => None,
        }
    }
}

fn contains(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_returns_the_first_matching_target() {
        let mut hits = HitMap::default();
        hits.register(Rect::new(0, 0, 10, 1), Action::Quit);
        hits.register(Rect::new(0, 1, 10, 3), Action::ClickRow(2));
        assert_eq!(hits.hit(9, 0), Some(Action::Quit));
        assert_eq!(hits.hit(0, 3), Some(Action::ClickRow(2)));
        assert_eq!(hits.hit(10, 0), None);
        assert_eq!(hits.row_at(5, 2), Some(2));
        assert_eq!(hits.row_at(5, 0), None);
    }

    #[test]
    fn scrollbar_jump_maps_track_position_to_rows() {
        let mut hits = HitMap::default();
        hits.set_scrollbar(Rect::new(79, 1, 1, 21), 100);
        assert_eq!(hits.scrollbar_jump(79, 1), Some(Action::SelectRow(0)));
        assert_eq!(hits.scrollbar_jump(79, 21), Some(Action::SelectRow(99)));
        assert_eq!(hits.scrollbar_jump(79, 11), Some(Action::SelectRow(50)));
        assert_eq!(hits.scrollbar_jump(78, 11), None);
        assert_eq!(hits.scrollbar_jump(79, 0), None);
    }

    #[test]
    fn popup_containment_and_clear() {
        let mut hits = HitMap::default();
        hits.set_popup(Rect::new(5, 5, 10, 5));
        assert!(hits.popup_active());
        assert!(hits.popup_contains(5, 5));
        assert!(!hits.popup_contains(4, 5));
        hits.clear();
        assert!(!hits.popup_active());
    }
}
