use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::Command;

pub fn map_event(key: KeyEvent) -> Option<Command> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) => Some(Command::Quit),
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Command::Quit),
        (KeyCode::Esc, _) => Some(Command::Quit),
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => Some(Command::MoveUp),
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => Some(Command::MoveDown),
        (KeyCode::Char('l'), _) | (KeyCode::Right, _) => Some(Command::ExpandOrOpen),
        (KeyCode::Enter, _) => Some(Command::ActivateSelected),
        (KeyCode::Char('h'), _) | (KeyCode::Left, _) => Some(Command::Collapse),
        (KeyCode::Char('r'), _) => Some(Command::RefreshGit),
        (KeyCode::Tab, _) => Some(Command::ToggleTreeMode),
        (KeyCode::Char('?'), _) | (KeyCode::F(1), _) => Some(Command::ToggleHelp),
        (KeyCode::Char('t'), _) => Some(Command::ToggleHelpLanguage),
        (KeyCode::Char('c'), _) => Some(Command::CopyAtRelativePath),
        (KeyCode::Char('o'), _) => Some(Command::OpenInFinder),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::map_event;
    use crate::app::Command;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    #[test]
    fn esc_maps_to_quit() {
        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(matches!(map_event(event), Some(Command::Quit)));
    }

    #[test]
    fn c_maps_to_copy_at_relative_path() {
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        assert!(matches!(
            map_event(event),
            Some(Command::CopyAtRelativePath)
        ));
    }

    #[test]
    fn enter_maps_to_activate_selected() {
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(matches!(map_event(event), Some(Command::ActivateSelected)));
    }

    #[test]
    fn ctrl_c_maps_to_quit() {
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(matches!(map_event(event), Some(Command::Quit)));
    }

    #[test]
    fn t_maps_to_toggle_help_language() {
        let event = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
        assert!(matches!(
            map_event(event),
            Some(Command::ToggleHelpLanguage)
        ));
    }

    #[test]
    fn tab_maps_to_toggle_tree_mode() {
        let event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert!(matches!(map_event(event), Some(Command::ToggleTreeMode)));
    }

    #[test]
    fn repeat_is_ignored() {
        let mut event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        event.kind = KeyEventKind::Repeat;

        assert!(map_event(event).is_none());
    }
}
