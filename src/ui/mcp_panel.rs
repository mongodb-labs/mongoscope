use iced::{
    widget::{button, column, container, mouse_area, row, text},
    Alignment, Background, Border, Color, Element, Font, Length, Padding,
};
use crate::theme::Palette;

#[derive(Debug, Clone, PartialEq)]
pub enum McpServerState {
    Stopped,
    Starting,
    Running { port: u16 },
}

#[derive(Debug, Clone)]
pub struct McpPanelState {
    pub open: bool,
    pub server: McpServerState,
}

#[derive(Debug, Clone)]
pub enum McpMsg {
    Toggle,
    StartStop,
    Started,
    CopyConfig,
    Noop,
}

impl McpPanelState {
    pub fn new() -> Self {
        Self { open: false, server: McpServerState::Stopped }
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Returns true if caller should fire async start task.
    pub fn begin_start(&mut self) -> bool {
        if matches!(self.server, McpServerState::Stopped) {
            self.server = McpServerState::Starting;
            true
        } else {
            false
        }
    }

    pub fn on_started(&mut self) {
        if matches!(self.server, McpServerState::Starting) {
            self.server = McpServerState::Running { port: 3717 };
        }
    }

    pub fn stop(&mut self) {
        if matches!(self.server, McpServerState::Running { .. }) {
            self.server = McpServerState::Stopped;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_panel_is_closed_and_stopped() {
        let p = McpPanelState::new();
        assert!(!p.open);
        assert_eq!(p.server, McpServerState::Stopped);
    }

    #[test]
    fn toggle_opens_and_closes() {
        let mut p = McpPanelState::new();
        p.toggle(); assert!(p.open);
        p.toggle(); assert!(!p.open);
    }

    #[test]
    fn begin_start_transitions_to_starting_and_returns_true() {
        let mut p = McpPanelState::new();
        let fired = p.begin_start();
        assert!(fired);
        assert_eq!(p.server, McpServerState::Starting);
    }

    #[test]
    fn begin_start_while_starting_does_nothing() {
        let mut p = McpPanelState::new();
        p.begin_start();
        let fired = p.begin_start();
        assert!(!fired);
        assert_eq!(p.server, McpServerState::Starting);
    }

    #[test]
    fn on_started_transitions_to_running_port_3717() {
        let mut p = McpPanelState::new();
        p.begin_start();
        p.on_started();
        assert_eq!(p.server, McpServerState::Running { port: 3717 });
    }

    #[test]
    fn stop_transitions_running_to_stopped() {
        let mut p = McpPanelState::new();
        p.begin_start();
        p.on_started();
        p.stop();
        assert_eq!(p.server, McpServerState::Stopped);
    }

    #[test]
    fn stop_while_stopped_does_nothing() {
        let mut p = McpPanelState::new();
        p.stop();
        assert_eq!(p.server, McpServerState::Stopped);
    }
}
