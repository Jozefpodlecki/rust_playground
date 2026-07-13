use imgui::{ColorStackToken, Condition, StyleColor, StyleStackToken, StyleVar, Ui};

fn push_style(ui: &Ui) -> ([StyleStackToken; 5], [ColorStackToken; 7]) {
    let style_tokens = [
        ui.push_style_var(StyleVar::WindowRounding(8.0)),
        ui.push_style_var(StyleVar::FrameRounding(4.0)),
        ui.push_style_var(StyleVar::WindowPadding([10.0, 10.0])),
        ui.push_style_var(StyleVar::FramePadding([8.0, 6.0])),
        ui.push_style_var(StyleVar::ItemSpacing([10.0, 8.0])),
    ];
    
    let color_tokens = [
        ui.push_style_color(StyleColor::WindowBg, [0.12, 0.12, 0.15, 1.0]),
        ui.push_style_color(StyleColor::Button, [0.25, 0.45, 0.75, 1.0]),
        ui.push_style_color(StyleColor::ButtonHovered, [0.35, 0.55, 0.85, 1.0]),
        ui.push_style_color(StyleColor::ButtonActive, [0.15, 0.35, 0.65, 1.0]),
        ui.push_style_color(StyleColor::Text, [0.9, 0.9, 0.95, 1.0]),
        ui.push_style_color(StyleColor::TitleBg, [0.18, 0.22, 0.3, 1.0]),
        ui.push_style_color(StyleColor::TitleBgActive, [0.22, 0.28, 0.4, 1.0]),
    ];
    
    (style_tokens, color_tokens)
}

fn pop_style(style_tokens: [StyleStackToken; 5], color_tokens: [ColorStackToken; 7]) {
    // for token in color_tokens.into_iter().rev() {
    //     token.pop();
    // }
    // for token in style_tokens.into_iter().rev() {
    //     token.pop();
    // }
}

fn render_sidebar(ui: &Ui) {
    ui.child_window("Sidebar")
        .size([200.0, 0.0])
        .border(true)
        .build(|| {
            ui.text_colored([0.6, 0.8, 1.0, 1.0], "Navigation");
            ui.separator();
            ui.dummy([0.0, 10.0]);
            
            if ui.button_with_size("Home", [180.0, 30.0]) {
                println!("Home clicked");
            }
            if ui.button_with_size("Settings", [180.0, 30.0]) {
                println!("Settings clicked");
            }
            if ui.button_with_size("About", [180.0, 30.0]) {
                println!("About clicked");
            }
            
            ui.dummy([0.0, 20.0]);
            ui.separator();
            ui.text_colored([0.5, 0.5, 0.5, 1.0], "Status:");
            ui.text("Ready");
        });
}

fn render_content(ui: &Ui) {
    ui.child_window("Content")
        .size([0.0, 0.0])
        .border(true)
        .build(|| {
            ui.text_colored([0.6, 0.8, 1.0, 1.0], "Main Content");
            ui.separator();
            ui.dummy([0.0, 10.0]);
            
            ui.text("Welcome to the main content area!");
            ui.text("This is where your main content goes.");
            
            ui.dummy([0.0, 15.0]);
            ui.separator();
            ui.dummy([0.0, 10.0]);
            
            if ui.button_with_size("Click me!", [120.0, 32.0]) {
                println!("Button clicked!");
            }
        });
}

pub fn render_main_window(ui: &Ui) {
    let (style_tokens, color_tokens) = push_style(ui);

    ui.window("Test")
        .size([800.0, 600.0], Condition::FirstUseEver)
        .position([100.0, 100.0], Condition::FirstUseEver)
        .resizable(true)
        .movable(true)
        .collapsible(true)
        .build(|| {
            render_sidebar(ui);
            ui.same_line();
            render_content(ui);
        });
    
    pop_style(style_tokens, color_tokens);
}