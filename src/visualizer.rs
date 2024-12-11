use std::sync::Arc;
use std::sync::Mutex;

use egui::pos2;
use egui::vec2;
use egui::Align2;
use egui::Window;

use crate::adsr::ADSR;
use crate::filter::Filter;
use crate::filter::FilterTypes;
use crate::filter::HighPassFilter;
use crate::filter::LowPassFilter;
use crate::waveform::Wave;
use crate::waveform::Waveform;
use crate::NoteState;

pub struct WindowState {
    adsr: bool,
    wave: bool,
    filters: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        WindowState {
            adsr: true,
            wave: true,
            filters: true,
        }
    }
}

pub struct Visualizer {
    pub audio_data: Arc<Mutex<Vec<f32>>>,
    pub adsr: Arc<Mutex<ADSR>>,
    pub wave: Arc<Mutex<Wave>>,
    pub filters: Arc<Mutex<Vec<Box<dyn Filter + Send + Sync>>>>,
    pub filter_select: FilterTypes,
    pub filter_cutoff: f32,
    pub window_state: WindowState,
    pub note_state: Arc<Mutex<NoteState>>,
}

impl eframe::App for Visualizer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("Top").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Window", |ui| {
                    if ui.button("ADSR").clicked() {
                        self.window_state.adsr = !self.window_state.adsr;
                    }
                    if ui.button("Waveform").clicked() {
                        self.window_state.wave = !self.window_state.wave;
                    }
                    if ui.button("Filters").clicked() {
                        self.window_state.filters = !self.window_state.filters;
                    }
                })
            });
        });

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
        let mut wave = self.wave.lock().unwrap();
        let mut filters = self.filters.lock().unwrap();

        Window::new("ADSR")
            .open(&mut self.window_state.adsr)
            .show(ctx, |ui| {
                ui.label("ADSR");
                ui.add(egui::Slider::new(&mut adsr.attack, 0.0..=2.0).text("Attack: "));
                ui.add(egui::Slider::new(&mut adsr.decay, 0.0..=2.0).text("Decay: "));
                ui.add(egui::Slider::new(&mut adsr.sustain, 0.0..=1.0).text("Sustain: "));
                ui.add(egui::Slider::new(&mut adsr.release, 0.0..=2.0).text("Release: "));
            });

        Window::new("Waveform")
            .open(&mut self.window_state.wave)
            .show(ctx, |ui| {
                ui.label("Waveform");
                egui::ComboBox::from_label("Type")
                    .selected_text(format!("{:?}", &wave.waveform))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut wave.waveform, Waveform::Sine, "Sine");
                        ui.selectable_value(&mut wave.waveform, Waveform::Square, "Square");
                        ui.selectable_value(&mut wave.waveform, Waveform::Sawtooth, "Sawtooth");
                    });
                ui.label("Frequency");
                ui.add(egui::Slider::new(&mut wave.frequency, 0.0..=1000.0));
            });

        Window::new("Filters")
            .open(&mut self.window_state.filters)
            .show(ctx, |ui| {
                egui::ComboBox::from_label("Type")
                    .selected_text(format!("{:?}", &self.filter_select))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.filter_select,
                            FilterTypes::High,
                            "High pass",
                        );
                        ui.selectable_value(&mut self.filter_select, FilterTypes::Low, "Low pass");
                    });
                ui.add(egui::Slider::new(&mut self.filter_cutoff, 0.0..=1000.0));
                if ui.add(egui::Button::new("Add Filter")).clicked() {
                    let filter: Box<dyn Filter + Send + Sync> = match self.filter_select {
                        FilterTypes::High => {
                            Box::new(HighPassFilter::new(self.filter_cutoff, wave.sample_rate))
                        }
                        FilterTypes::Low => {
                            Box::new(LowPassFilter::new(self.filter_cutoff, wave.sample_rate))
                        }
                    };
                    filters.push(filter);
                }

                let mut to_remove = vec![];

                ui.collapsing("Filters: ", |ui| {
                    for (i, _) in filters.iter().enumerate() {
                        if ui.add(egui::Button::new(format!("{}: ", i))).clicked() {
                            to_remove.push(i);
                        }
                    }
                });
                to_remove.iter().for_each(|i| {
                    filters.remove(*i);
                });
            });

        let mut note_state = self.note_state.lock().unwrap();

        if ctx.input(|i| i.key_pressed(egui::Key::A)) {
            note_state.note_on();
        }

        if ctx.input(|i| i.key_released(egui::Key::A)) {
            note_state.note_off();
        }
        
        egui::Area::new(egui::Id::new("DEBUG"))
        .anchor(Align2::RIGHT_BOTTOM, vec2(0.0,0.0))
        .show(ctx, |ui| {
            ui.label(format!("Note State: {:?}", *note_state));
        });

        ctx.request_repaint();
    }
}
