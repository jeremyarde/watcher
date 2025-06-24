use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptType {
    ForegroundApp,
    ForegroundWindowTitle,
    SafariTabInfo,
    ChromeTabInfo,
    WifiNetwork,
    IdleTime,
    LockScreen,
    StartPomodoroReminder,
    QuitDistractingApps,
}

pub fn load_scripts() -> HashMap<ScriptType, String> {
    let mut map = HashMap::new();

    map.insert(
        ScriptType::ForegroundApp,
        String::from(
            r#"
        tell application \"System Events\"
            get name of first application process whose frontmost is true
        end tell
    "#,
        ),
    );

    map.insert(
        ScriptType::ForegroundWindowTitle,
        String::from(
            r#"
        tell application \"System Events\"
            set frontApp to first application process whose frontmost is true
            try
                get name of front window of frontApp
            on error
                return ""
            end try
        end tell
    "#,
        ),
    );

    map.insert(
        ScriptType::SafariTabInfo,
        String::from(
            r#"
        try
            tell application \"Safari\"
                set tabName to name of current tab of front window
                set tabURL to URL of current tab of front window
                return tabName & \" | \" & tabURL
            end tell
        on error
            return ""
        end try
    "#,
        ),
    );

    map.insert(
        ScriptType::ChromeTabInfo,
        String::from(
            r#"
        tell application \"Google Chrome\"
            set tabName to title of active tab of front window
            set tabURL to URL of active tab of front window
        end tell
        return tabName & \" | \" & tabURL
    "#,
        ),
    );

    map.insert(
        ScriptType::WifiNetwork,
        String::from(
            r#"
        do shell script \"networksetup -getairportnetwork en0 | cut -d ':' -f2 | sed 's/^ *//'\"
    "#,
        ),
    );

    map.insert(
        ScriptType::IdleTime,
        String::from(
            r#"
        do shell script \"ioreg -c IOHIDSystem | awk '/HIDIdleTime/ {print $NF/1000000000; exit}'\"
    "#,
        ),
    );

    map.insert(ScriptType::LockScreen, String::from(r#"
        do shell script "/System/Library/CoreServices/Menu\\ Extras/User.menu/Contents/Resources/CGSession -suspend"
    "#));

    map.insert(
        ScriptType::StartPomodoroReminder,
        String::from(
            r#"
        tell application \"Reminders\"
            set dueDate to (current date) + (25 * minutes)
            make new reminder with properties {name:\"Take a break\", remind me date:dueDate}
        end tell
    "#,
        ),
    );

    map.insert(
        ScriptType::QuitDistractingApps,
        String::from(
            r#"
        try
            tell application \"Slack\" to quit
        end try
        try
            tell application \"Discord\" to quit
        end try
    "#,
        ),
    );

    map
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    #[test]
    fn test_load_scripts() {
        let scripts = load_scripts();
        assert!(!scripts.is_empty());
    }

    #[test]
    fn test_all_scripts_are_valid() {
        let scripts = load_scripts();
        for (script_type, script) in &scripts {
            let output = Command::new("osascript")
                .arg("-e")
                .arg(script)
                .output()
                .expect("Failed to execute script");
            println!(
                "Script {:?}: {}",
                script_type,
                String::from_utf8_lossy(&output.stdout)
            );
        }
    }
}
