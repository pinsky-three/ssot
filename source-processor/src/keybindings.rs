use egui::{Context, Key};

/// Commands that can be triggered by keyboard shortcuts
#[derive(Debug, Clone, Copy)]
pub enum Command {
    ToggleDebug,
    OpenKeybindings,
    CloseKeybindings,
    ResetAll,
    ToggleNavMode,
    FitToScreenOnce,
}

/// Dispatch keyboard events to commands
pub fn dispatch(ctx: &Context) -> Vec<Command> {
    let mut cmds = Vec::new();
    let mut pressed_h = false;
    let mut pressed_shift_slash = false;

    ctx.input(|i| {
        // Track special key combinations for keybindings modal
        for ev in &i.events {
            match ev {
                egui::Event::Key {
                    key,
                    pressed,
                    modifiers,
                    ..
                } => {
                    if *pressed && !modifiers.any() && *key == Key::H {
                        pressed_h = true;
                    }
                    if *pressed && *key == Key::Slash && modifiers.shift {
                        pressed_shift_slash = true;
                    }
                }
                egui::Event::Text(t) => {
                    if t == "?" {
                        pressed_shift_slash = true;
                    }
                    if t.eq_ignore_ascii_case("h") {
                        pressed_h = true;
                    }
                }
                _ => {}
            }
        }

        // Backspace: reset to defaults
        if i.key_pressed(Key::Backspace) && !i.modifiers.any() {
            cmds.push(Command::ResetAll);
        }

        // Space: navigation controls
        if i.key_pressed(Key::Space) {
            if i.modifiers.ctrl {
                cmds.push(Command::ToggleNavMode);
            } else if !i.modifiers.any() {
                cmds.push(Command::FitToScreenOnce);
            }
        }

        // Escape: close modals
        if i.key_pressed(Key::Escape) {
            cmds.push(Command::CloseKeybindings);
        }

        // Process other key events
        for ev in &i.events {
            if let egui::Event::Key {
                key,
                pressed,
                modifiers,
                ..
            } = ev
            {
                if !pressed {
                    continue;
                }
                match key {
                    Key::D if !modifiers.any() => {
                        cmds.push(Command::ToggleDebug);
                    }
                    _ => {}
                }
            }
        }
    });

    // H or ? opens keybindings
    if pressed_h || pressed_shift_slash {
        cmds.push(Command::OpenKeybindings);
    }

    cmds
}

