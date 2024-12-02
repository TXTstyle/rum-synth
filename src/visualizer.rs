use std::sync::Arc;
use std::sync::Mutex;

use crate::adsr::ADSR;
use crate::waveform;
use crate::waveform::Waveform;

pub struct Visualizer {
    pub audio_data: Arc<Mutex<Vec<f32>>>,
    pub adsr: Arc<Mutex<ADSR>>,
    pub waveform: Arc<Mutex<Waveform>>,
}

impl eframe::App for Visualizer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Waveform Visualizer");

            let painter = ui.painter();

            // Get the latest audio data
            let audio_data = self.audio_data.lock().unwrap();
            let wave_samples: Vec<f32> = audio_data.iter().cloned().take(500).collect();

            let canvas_rect = ui.available_rect_before_wrap();
            let canvas_width = canvas_rect.width();
            let canvas_height = canvas_rect.height();

            let center_y = canvas_rect.top() + canvas_height / 2.0;

            // Draw the waveform
            if !wave_samples.is_empty() {
                let points: Vec<egui::Pos2> = wave_samples
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| {
                        let x = canvas_rect.left()
                            + i as f32 * canvas_width / wave_samples.len() as f32;
                        let y = center_y - v * canvas_height / 2.0;
                        egui::pos2(x, y)
                    })
                    .collect();

                for window in points.windows(2) {
                    if let [start, end] = window {
                        painter.line_segment(
                            [*start, *end],
                            egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
                        );
                    }
                }
            }
        });

        let mut adsr = self.adsr.lock().unwrap();
        let mut waveform = self.waveform.lock().unwrap();

        egui::Window::new("ADSR").show(ctx, |ui| {
            ui.label("ADSR");
            ui.add(egui::Slider::new(&mut adsr.attack, 0.0..=2.0).text("Attack: "));
            ui.add(egui::Slider::new(&mut adsr.decay, 0.0..=2.0).text("Decay: "));
            ui.add(egui::Slider::new(&mut adsr.sustain, 0.0..=1.0).text("Sustain: "));
            ui.add(egui::Slider::new(&mut adsr.release, 0.0..=2.0).text("Release: "));
        });

        egui::Window::new("Waveform").show(ctx, |ui| {
            ui.label("Waveform");
            egui::ComboBox::from_label("Type: ")
                .selected_text(format!("{:?}", &waveform))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut *waveform, Waveform::Sine, "Sine");
                    ui.selectable_value(&mut *waveform, Waveform::Square, "Square");
                    ui.selectable_value(&mut *waveform, Waveform::Sawtooth, "Sawtooth");
                });
        });


        ctx.request_repaint();
    }
}
