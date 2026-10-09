use std::collections::VecDeque;

use us_hr_core::DeviceSettings;

pub(crate) struct UndoHistory {
    states: VecDeque<DeviceSettings>,
    capacity: usize,
}

impl UndoHistory {
    pub(crate) const fn new(capacity: usize) -> Self {
        Self {
            states: VecDeque::new(),
            capacity,
        }
    }

    pub(crate) fn record(&mut self, settings: DeviceSettings) {
        if self.capacity == 0 || self.states.back() == Some(&settings) {
            return;
        }
        if self.states.len() == self.capacity {
            let _ = self.states.pop_front();
        }
        self.states.push_back(settings);
    }

    pub(crate) fn pop(&mut self) -> Option<DeviceSettings> {
        self.states.pop_back()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    pub(crate) fn clear(&mut self) {
        self.states.clear();
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.states.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_history_and_skips_duplicates() {
        let mut history = UndoHistory::new(2);
        let mut first = DeviceSettings::default();
        let mut second = first;
        let mut third = first;
        second.broadcast_volume = 80;
        third.broadcast_volume = 40;

        history.record(first);
        history.record(first);
        history.record(second);
        history.record(third);

        assert_eq!(history.len(), 2);
        assert_eq!(history.pop(), Some(third));
        assert_eq!(history.pop(), Some(second));
        assert_eq!(history.pop(), None);

        first.broadcast_volume = 1;
        history.record(first);
        assert!(!history.is_empty());
        history.clear();
        assert!(history.is_empty());
    }

    #[test]
    fn zero_capacity_never_records_state() {
        let mut history = UndoHistory::new(0);
        history.record(DeviceSettings::default());

        assert!(history.is_empty());
        assert_eq!(history.pop(), None);
    }

    #[test]
    fn retains_non_adjacent_states_for_multi_step_undo() {
        let mut history = UndoHistory::new(3);
        let first = DeviceSettings::default();
        let mut second = first;
        second.broadcast_volume = 42;

        history.record(first);
        history.record(second);
        history.record(first);

        assert_eq!(history.len(), 3);
        assert_eq!(history.pop(), Some(first));
        assert_eq!(history.pop(), Some(second));
        assert_eq!(history.pop(), Some(first));
    }
}
