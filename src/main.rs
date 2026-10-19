#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod csv_plan;
mod csv_recovery;
mod csv_ui;
mod csv_workspace;
#[cfg(test)]
mod csv_workspace_tests;
#[cfg(feature = "embedded-wordcraft")]
mod docx_validation;
mod engines;
mod preferences;
mod ui;
#[cfg(feature = "embedded-wordcraft")]
mod wordcraft_embed;

use eframe::egui;

fn setup_korean_font(ctx: &egui::Context) {
    let candidates = [
        r"C:\Windows\Fonts\malgun.ttf",
        r"C:\Windows\Fonts\malgunbd.ttf",
        "/usr/share/fonts/truetype/nanum/NanumGothic.ttf",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "korean-system".to_string(),
                egui::FontData::from_owned(bytes).into(),
            );
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.insert(0, "korean-system".to_string());
            }
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                family.insert(0, "korean-system".to_string());
            }
            ctx.set_fonts(fonts);
            break;
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ToolBox - Desktop Preview")
            .with_inner_size([1100.0, 740.0])
            .with_min_inner_size([780.0, 530.0]),
        ..Default::default()
    };
    eframe::run_native(
        "ToolBox",
        options,
        Box::new(|cc| {
            setup_korean_font(&cc.egui_ctx);
            Ok(Box::new(ui::ToolBoxApp::new()))
        }),
    )
}
