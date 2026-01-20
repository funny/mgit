use eframe::egui;

use mgit::utils::style_message::StyleMessage;
use mgit::Colour;

use crate::ui::style::text_color;

pub(crate) fn create_layout_jobs(messages: &[StyleMessage]) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();

    for message in messages {
        for span in &message.0 {
            let mut format = egui::TextFormat::default();
            format.font_id.size = 13.0;
            format.color = text_color::from(&span.style.map(|s| {
                match s {
                    // Map known ansi_term colors to mgit::Colour
                    s if format!("{:?}", s.foreground).contains("Red") => Colour::Red,
                    s if format!("{:?}", s.foreground).contains("Green") => Colour::Green,
                    s if format!("{:?}", s.foreground).contains("Blue") => Colour::Blue,
                    s if format!("{:?}", s.foreground).contains("Yellow") => Colour::Yellow,
                    s if format!("{:?}", s.foreground).contains("Purple") => Colour::Purple,
                    s if format!("{:?}", s.foreground).contains("White") => Colour::White,
                    s if format!("{:?}", s.foreground).contains("Black") => Colour::Black,
                    _ => Colour::Fixed(245), // Default to grey/light grey
                }
            }));

            job.append(&span.content, 0.0, format);
        }
    }
    job
}

pub(crate) fn create_layout_job(text: String, color: egui::Color32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let mut format = egui::TextFormat::default();
    format.font_id.size = 13.0;
    format.color = color;
    job.append(&text, 0.0, format);
    job
}

pub(crate) fn create_truncate_layout_job(
    text: String,
    color: egui::Color32,
) -> egui::text::LayoutJob {
    let mut format = egui::TextFormat::default();
    format.font_id.size = 13.0;
    format.color = color;

    egui::text::LayoutJob {
        sections: vec![egui::text::LayoutSection {
            leading_space: 0.0,
            byte_range: 0..text.len(),
            format: format,
        }],
        wrap: egui::epaint::text::TextWrapping {
            max_rows: 1,
            break_anywhere: true,
            ..Default::default()
        },
        break_on_newline: false,
        halign: egui::Align::RIGHT,
        justify: true,
        text,
        ..Default::default()
    }
}
