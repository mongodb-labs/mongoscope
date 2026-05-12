use crate::ui::sidebar::connections::ConnectionColor;

#[derive(Debug, Clone, PartialEq)]
pub enum DialogStep {
    Step1 { connecting: bool },
    Step2,
}

#[derive(Debug, Clone)]
pub struct ConnectionDialogState {
    pub step: DialogStep,
    pub uri: String,
    pub name: String,
    pub color: ConnectionColor,
    pub error: Option<String>,
    pub proxy_port: u16,
}

impl ConnectionDialogState {
    pub fn new() -> Self {
        Self {
            step: DialogStep::Step1 { connecting: false },
            uri: "mongodb://localhost:27017/".into(),
            name: String::new(),
            color: ConnectionColor::None,
            error: None,
            proxy_port: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_dialog_starts_step1_idle() {
        let d = ConnectionDialogState::new();
        assert_eq!(d.step, DialogStep::Step1 { connecting: false });
    }

    #[test]
    fn new_dialog_has_default_uri() {
        let d = ConnectionDialogState::new();
        assert_eq!(d.uri, "mongodb://localhost:27017/");
    }

    #[test]
    fn new_dialog_error_is_none() {
        let d = ConnectionDialogState::new();
        assert!(d.error.is_none());
    }

    #[test]
    fn new_dialog_proxy_port_zero() {
        let d = ConnectionDialogState::new();
        assert_eq!(d.proxy_port, 0);
    }
}
