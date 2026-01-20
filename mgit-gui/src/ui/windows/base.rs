use eframe::egui;
use eframe::egui::{Ui, Vec2};

pub trait View {
    fn ui(&mut self, ui: &mut Ui);
}

pub trait WindowBase: View {
    fn name(&self) -> String;

    fn width(&self) -> f32;

    fn height(&self) -> f32;

    fn default_pos(&self, screen_rect: &Vec2) -> [f32; 2];

    #[allow(unused_variables)]
    fn before_show(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame, open: &mut bool) {}

    fn show(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame, open: &mut bool) {
        let default_pos = self.default_pos(&ctx.viewport_rect().size());

        self.before_show(ctx, eframe, open);
        egui::Window::new(self.name())
            .fixed_pos(default_pos)
            .fixed_size([self.width(), self.height()])
            .collapsible(false)
            .open(open)
            .show(ctx, |ui| {
                self.ui(ui);
            });
        self.after_show(ctx, eframe, open);
    }

    #[allow(unused_variables)]
    fn after_show(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame, open: &mut bool) {}
}
