use std::fs;
use std::path::Path;
use slint::ComponentHandle;

slint::slint! {
    import { Button, LineEdit, ScrollView } from "std-widgets.slint";

    export struct Shortcut {
        action: string,
        keybind: string,
        keys: [string],
    }

    export component AppWindow inherits Window {
        background: #09090ddd;
        no-frame: true;
        always-on-top: true;
        width: 950px;
        height: 520px;

        in property <[Shortcut]> shortcuts;
        out property <string> filter-text;

        callback filter-changed(string);
        callback close-window();

        FocusScope {
            key-pressed(event) => {
                if (event.text == "\u{001b}") { // Escape key
                    root.close-window();
                    return accept;
                }
                reject
            }

            VerticalLayout {
                padding: 20px;
                spacing: 12px;

                // Top Header area (Only search bar)
                HorizontalLayout {
                    alignment: LayoutAlignment.center;

                    search-input := LineEdit {
                        placeholder-text: "Type to search shortcuts...";
                        font-size: 14px;
                        height: 42px;
                        width: 100%; // Spans full width
                        edited(text) => {
                            root.filter-text = text;
                            root.filter-changed(text);
                        }
                    }
                }

                // Shortcuts list container
                Rectangle {
                    background: #161920;
                    border-radius: 8px;
                    border-width: 1px;
                    border-color: #2d3139;

                    VerticalLayout {
                        padding: 10px;
                        spacing: 4px;

                        // Header row
                        HorizontalLayout {
                            padding-left: 12px;
                            padding-right: 12px;
                            padding-top: 8px;
                            padding-bottom: 8px;

                            Text {
                                text: "Action";
                                font-size: 13px;
                                color: #7daea3;
                                font-weight: 700;
                                width: 580px;
                            }
                            Text {
                                text: "Shortcut";
                                font-size: 13px;
                                color: #7daea3;
                                font-weight: 700;
                                horizontal-alignment: right;
                            }
                        }

                        // Scrollable List
                        ScrollView {
                            VerticalLayout {
                                spacing: 8px;
                                for s in root.shortcuts : Rectangle {
                                    background: ta.has-hover ? #222735 : #1c202a;
                                    border-radius: 6px;

                                    ta := TouchArea {}

                                    HorizontalLayout {
                                        padding-left: 12px;
                                        padding-right: 12px;
                                        padding-top: 8px;
                                        padding-bottom: 8px;
                                        alignment: space-between;

                                        // Action name column
                                        Text {
                                            text: s.action;
                                            color: ta.has-hover ? #ffffff : #f2f2f7;
                                            font-size: 13px;
                                            vertical-alignment: center;
                                            width: 580px;
                                            wrap: word-wrap;
                                        }

                                        // Individual pills for the keys
                                        HorizontalLayout {
                                            spacing: 5px;
                                            alignment: LayoutAlignment.center;

                                            for k in s.keys : Rectangle {
                                                background: #2d3139;
                                                border-radius: 4px;
                                                border-width: 1px;
                                                border-color: #454c59;
                                                height: 22px;

                                                HorizontalLayout {
                                                    padding-left: 6px;
                                                    padding-right: 6px;
                                                    alignment: LayoutAlignment.center;

                                                    Text {
                                                        text: k;
                                                        color: #a9b665;
                                                        font-family: "JetBrains Mono";
                                                        font-size: 11px;
                                                        font-weight: 700;
                                                        vertical-alignment: center;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let lock_path = Path::new("/tmp/kwin-shortcut-overlay.lock");

    // --- Single Instance Check ---
    if lock_path.exists() {
        if let Ok(pid_str) = fs::read_to_string(lock_path) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                // Verify the process actually exists right now
                let process_exists = std::path::Path::new(&format!("/proc/{}", pid)).exists();
                if process_exists {
                    // It is running, terminate it
                    let _ = std::process::Command::new("kill")
                        .arg(pid.to_string())
                        .output();

                    let _ = fs::remove_file(lock_path);
                    return Ok(());
                } else {
                    // Stale lock file, clean it up and continue
                    let _ = fs::remove_file(lock_path);
                }
            }
        }
    }

    // First instance: write our PID
    let current_pid = std::process::id();
    let _ = fs::write(lock_path, current_pid.to_string());

    let main_window = AppWindow::new()?;

    // --- Loading config ---
    let home = std::env::var("HOME").unwrap_or_default();
    let config_path = Path::new(&home).join(".config/kglobalshortcutsrc");
    let mut shortcuts = Vec::new();

    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(config_path) {
            let mut in_kwin = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed == "[kwin]" {
                    in_kwin = true;
                    continue;
                } else if trimmed.starts_with('[') && in_kwin {
                    break;
                }

                if in_kwin && trimmed.contains('=') {
                    let parts: Vec<&str> = trimmed.split('=').collect();
                    if parts.len() == 2 && !parts[1].is_empty() {
                        let action = parts[0].trim().to_string();
                        let current_bind = parts[1].split(',').next().unwrap_or("").trim();
                        let raw_keybind = current_bind.split("\\t").next().unwrap_or("").trim();
                        let cleaned_keybind = raw_keybind.replace("Meta", "Super");

                        if !cleaned_keybind.is_empty() && cleaned_keybind != "none" {
                            let keys: Vec<slint::SharedString> = cleaned_keybind
                                .split('+')
                                .map(|k| slint::SharedString::from(k.trim()))
                                .collect();

                            shortcuts.push(Shortcut {
                                action: slint::SharedString::from(action),
                                keybind: slint::SharedString::from(cleaned_keybind),
                                keys: std::rc::Rc::new(slint::VecModel::from(keys)).into(),
                            });
                        }
                    }
                }
            }
        }
    }

    shortcuts.sort_by(|a, b| a.action.cmp(&b.action));
    let initial_shortcuts = shortcuts.clone();

    let model = std::rc::Rc::new(slint::VecModel::from(shortcuts));
    main_window.set_shortcuts(model.clone().into());

    let weak_window = main_window.as_weak();
    main_window.on_filter_changed(move |text| {
        if let Some(win) = weak_window.upgrade() {
            let filtered: Vec<Shortcut> = initial_shortcuts.iter()
                .filter(|s| s.action.to_lowercase().contains(&text.to_lowercase()))
                .cloned()
                .collect();
            win.set_shortcuts(std::rc::Rc::new(slint::VecModel::from(filtered)).into());
        }
    });

    let weak_window_close = main_window.as_weak();
    main_window.on_close_window(move || {
        if let Some(win) = weak_window_close.upgrade() {
            win.hide().unwrap();
        }
    });

    let result = main_window.run();
    let _ = fs::remove_file(lock_path);
    result
}
