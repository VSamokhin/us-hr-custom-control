use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Margin, RichText, Stroke, TextStyle,
};

pub(crate) const BACKGROUND: Color32 = Color32::from_rgb(10, 14, 20);
pub(crate) const SURFACE: Color32 = Color32::from_rgb(18, 24, 33);
pub(crate) const SURFACE_RAISED: Color32 = Color32::from_rgb(25, 33, 44);
pub(crate) const BORDER: Color32 = Color32::from_rgb(43, 55, 70);
pub(crate) const ACCENT: Color32 = Color32::from_rgb(67, 202, 190);
pub(crate) const ACCENT_DARK: Color32 = Color32::from_rgb(29, 104, 101);
pub(crate) const TEXT: Color32 = Color32::from_rgb(232, 238, 246);
pub(crate) const TEXT_MUTED: Color32 = Color32::from_rgb(137, 151, 169);
pub(crate) const SUCCESS: Color32 = Color32::from_rgb(91, 211, 145);
pub(crate) const SUCCESS_BG: Color32 = Color32::from_rgb(20, 54, 42);
pub(crate) const ERROR: Color32 = Color32::from_rgb(244, 112, 122);

pub(crate) fn configure(context: &egui::Context) {
    context.set_theme(egui::Theme::Dark);
    let mut style = (*context.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 9.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.interact_size.y = 34.0;
    style.spacing.slider_width = 210.0;
    style.spacing.slider_rail_height = 5.0;
    style.spacing.combo_width = 220.0;
    style.spacing.text_edit_width = 240.0;
    style.spacing.window_margin = Margin::same(0);
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(25.0, FontFamily::Proportional),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.0, FontFamily::Proportional),
    );

    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(TEXT);
    visuals.weak_text_color = Some(TEXT_MUTED);
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = SURFACE;
    visuals.window_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.faint_bg_color = SURFACE_RAISED;
    visuals.extreme_bg_color = Color32::from_rgb(8, 12, 18);
    visuals.text_edit_bg_color = Some(Color32::from_rgb(12, 17, 24));
    visuals.selection.bg_fill = ACCENT_DARK;
    visuals.selection.stroke = Stroke::new(1.0_f32, TEXT);
    visuals.hyperlink_color = ACCENT;
    visuals.warn_fg_color = Color32::from_rgb(241, 184, 91);
    visuals.error_fg_color = ERROR;
    visuals.slider_trailing_fill = true;
    visuals.button_frame = true;
    visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

    visuals.widgets.noninteractive.bg_fill = SURFACE;
    visuals.widgets.noninteractive.weak_bg_fill = SURFACE;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT_MUTED);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(7);

    visuals.widgets.inactive.bg_fill = SURFACE_RAISED;
    visuals.widgets.inactive.weak_bg_fill = SURFACE_RAISED;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(7);

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(34, 45, 59);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(34, 45, 59);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT_DARK);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(7);

    visuals.widgets.active.bg_fill = ACCENT_DARK;
    visuals.widgets.active.weak_bg_fill = ACCENT_DARK;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, TEXT);
    visuals.widgets.active.corner_radius = CornerRadius::same(7);

    visuals.widgets.open = visuals.widgets.active;
    style.visuals = visuals;
    context.set_style_of(egui::Theme::Dark, style);
}

pub(crate) fn card<R>(
    ui: &mut egui::Ui,
    eyebrow: &str,
    title: &str,
    subtitle: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(18))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                RichText::new(eyebrow.to_uppercase())
                    .size(10.0)
                    .strong()
                    .color(ACCENT),
            );
            ui.label(RichText::new(title).size(19.0).strong().color(TEXT));
            ui.label(RichText::new(subtitle).size(12.5).color(TEXT_MUTED));
            ui.add_space(8.0);
            add_contents(ui)
        })
        .inner
}

pub(crate) fn metric(ui: &mut egui::Ui, label: &str, value: &str, accent: Color32) {
    egui::Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 11))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                RichText::new(label.to_uppercase())
                    .size(9.5)
                    .strong()
                    .color(TEXT_MUTED),
            );
            ui.label(RichText::new(value).size(16.0).strong().color(accent));
        });
}

pub(crate) fn field_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(10.0)
            .strong()
            .color(TEXT_MUTED),
    );
}
