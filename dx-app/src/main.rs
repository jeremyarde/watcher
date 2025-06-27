use humantime::format_duration;
use std::time::{Duration, Instant};

use dioxus::prelude::*;
use dioxus_desktop::{
    Config, LogicalSize, WindowBuilder,
    tao::platform::macos::WindowBuilderExtMacOS,
    wry::dpi::{PhysicalSize, Size},
};

fn format_instant(instant: &Instant) -> String {
    // This will just show seconds ago for now, but you could use chrono for real timestamps
    let now = Instant::now();
    let duration = now.saturating_duration_since(*instant);
    format!("{} ago", format_duration(duration))
}

fn main() {
    let windowbuilder = WindowBuilder::new()
        // .with_decorations(false)
        .with_always_on_top(true)
        .with_movable_by_window_background(true)
        .with_fullsize_content_view(true)
        .with_resizable(true)
        // .with_max_inner_size(LogicalSize::new(200.0, 50.0));
        .with_inner_size(LogicalSize::new(400, 200));

    let appstate = background::create_process();

    let config = Config::default().with_window(windowbuilder);

    // dioxus::launch(app, config);

    LaunchBuilder::desktop()
        .with_cfg(config)
        .with_context(appstate)
        .launch(app);
}

fn app() -> Element {
    let show_dropdown = use_signal(|| false);
    let appstate = use_signal_sync(|| background::create_appstate());

    use_hook(|| {
        let mut appstate = appstate.clone();
        std::thread::spawn(move || {
            loop {
                let new_state = {
                    let mut state = appstate.write();
                    state.run_check();
                    state.clone()
                };
                appstate.set(new_state);
                std::thread::sleep(Duration::from_millis(2000));
            }
        });
    });

    let style = "
    body {
        margin: 0;
        padding: 0;
        font-family: sans-serif;
        overflow: visible;
    }
    .container {
        position: relative;
        margin: 40px;
        max-width: 600px;
    }
    .session-list {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }
    .session-item {
        display: flex;
        flex-direction: row;
        align-items: center;
        background: #f9f9f9;
        border-radius: 8px;
        box-shadow: 0 2px 8px rgba(0,0,0,0.04);
        padding: 16px;
        gap: 24px;
    }
    .session-item-time, .session-item-duration, .session-item-app, .session-item-window-title {
        font-size: 15px;
        color: #333;
    }
    .session-item-app {
        font-weight: bold;
        color: #2a4d8f;
    }
    .session-item-window-title {
        color: #666;
        font-style: italic;
    }
    .session-item-duration {
        color: #008060;
        font-weight: 500;
    }
";

    rsx! {
        style { style }

        div { class: "container",
            div { class: "session-list",
                if appstate().sessions.is_empty() {
                    div { class: "session-item",
                        div { class: "session-item-time", "No sessions yet" }
                    }
                }
                // for session in appstate().sessions.iter().rev() {
                //     // let duration = session.end_at.saturating_duration_since(session.start_at);
                //     div { class: "session-item",
                //         div { class: "session-item-app", "{session.app.trim()}" }
                //         div { class: "session-item-window-title", "{session.window_title.trim()}" }
                //         div { class: "session-item-time", "Started: {format_instant(&session.start_at)}" }
                //         div { class: "session-item-duration", "Duration: {format_duration(session.end_at.saturating_duration_since(session.start_at))}" }
                //     }
                // }

                // current app:
                div { class: "current-app", "{appstate().active_session.as_ref().unwrap().app.trim()}"}

                for (app, duration) in appstate().stats.total_time_per_app.iter() {
                    div { class: "session-item",
                        div { class: "session-item-app", "{app}" }
                        div { class: "session-item-duration", "{format_duration(*duration)}" }
                    }
                }
            }
        }
    }
}
