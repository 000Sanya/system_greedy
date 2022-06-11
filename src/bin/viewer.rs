use std::f32::consts::PI;
use iced::{Button, Canvas, canvas, Color, Column, Element, Length, Point, Radio, Rectangle, Row, Sandbox, Settings, Size, Text, Vector};
use iced::canvas::{Cursor, Event, Frame, Geometry, Path, Program};
use iced::canvas::event::Status;
use iced::mouse::ScrollDelta;
use itertools::Itertools;
use native_dialog::FileDialog;
use num_traits::Zero;
use ordered_float::OrderedFloat;
use quadtree_f32::{Item, ItemId, QuadTree};
use system_greedy::generators::LatticeGenerator;
use system_greedy::runner::State;
use system_greedy::system::System;

fn main() -> iced::Result {
    MainApp::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    SystemOpen,
    SystemUpdated,
    ViewUpdated,
    ReverseSpin(usize),
    FillModeChanged(ArrowFillMode),
}

pub struct MainApp {
    lattice_view_state: LatticeViewState,
    fill_mode: Option<ArrowFillMode>,
    system: Option<System>,
    open_button_state: iced::button::State
}

impl Sandbox for MainApp {
    type Message = AppMessage;

    fn new() -> Self {
        let system = LatticeGenerator::trimer(225., 700., 2, 2);

        Self {
            lattice_view_state: LatticeViewState::new(Some(&system)),
            fill_mode: Some(ArrowFillMode::Default),
            system: Some(system),
            open_button_state: iced::button::State::new(),
        }
    }

    fn title(&self) -> String {
        "System Viewer".to_owned()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            AppMessage::SystemOpen => {
                let path = FileDialog::new()
                    .set_location("~/Desktop")
                    .add_filter("MF System", &["mfsys"])
                    .show_open_single_file()
                    .unwrap();
                if let Some(path) = path {
                    self.system = Some(System::load_mfsys(&path));
                    self.lattice_view_state.reload(self.system.as_ref().unwrap());
                }
            }
            AppMessage::SystemUpdated => {}
            AppMessage::ViewUpdated => {}
            AppMessage::ReverseSpin(spin) => {
                if let Some(ref mut system) = self.system {
                    system.reverse_spin(spin)
                }
            }
            AppMessage::FillModeChanged(fill_mode) => {
                self.lattice_view_state.fill_mode = fill_mode;
                self.fill_mode = Some(fill_mode);
            }
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let (canvas, stats) = if let Some(ref system) = self.system {
            let canvas = vec![LatticeView::new(system, &mut self.lattice_view_state).view()];
            let stats = vec![
                Text::new(format!("Energy: {}", system.energy())).into()
            ];

            (canvas, stats)
        } else {
            (vec![], vec![])
        };

        Row::with_children(
            vec![
                Column::with_children(canvas)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into(),
                Column::new()
                    .push::<Element<'_, AppMessage>>(
                        Column::with_children(stats)
                            .into()
                    )
                    .push::<Element<'_, AppMessage>>(
                        Column::new()
                            .push::<Element<'_, AppMessage>>(
                                Radio::new(ArrowFillMode::Default, "Normal", self.fill_mode, |v| AppMessage::FillModeChanged(v)).into()
                            )
                            .push::<Element<'_, AppMessage>>(
                                Radio::new(ArrowFillMode::TopDown, "Top-down", self.fill_mode, |v| AppMessage::FillModeChanged(v)).into()
                            )
                            .push::<Element<'_, AppMessage>>(
                                Radio::new(ArrowFillMode::Energy, "Energy", self.fill_mode, |v| AppMessage::FillModeChanged(v)).into()
                            )
                            .push::<Element<'_, AppMessage>>(
                                Button::new(&mut self.open_button_state, Text::new("Open"))
                                    .on_press(AppMessage::SystemOpen)
                                    .into()
                            )
                            .into()
                    )
                    .width(Length::Units(300))
                    .height(Length::Fill)
                    .into(),
            ]
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ArrowFillMode {
    Default,
    TopDown,
    Energy,
}

impl ArrowFillMode {
    pub fn get_colors(&self, system: &System) -> Vec<Color> {
        match self {
            ArrowFillMode::Default => std::iter::repeat(Color::BLACK).take(system.size()).collect(),
            ArrowFillMode::TopDown => system.elements().iter()
                .enumerate()
                .map(|(i, element)| {
                    let mut dir = element.magn.map(|x| x.0);
                    if system.system_state()[i] {
                        dir *= -1.;
                    }
                    if dir.y.is_sign_positive() || dir.y.is_zero() && dir.x.is_sign_positive() {
                        Color::BLACK
                    } else {
                        Color::WHITE
                    }
                })
                .collect(),
            ArrowFillMode::Energy => {
                let (min, max) = system.row_energies().iter().copied().minmax_by_key(|x| OrderedFloat(*x))
                    .into_option().unwrap();

                system.row_energies().iter().copied()
                    .map(|energy| {
                        if energy.is_sign_positive() {
                            let percent = energy / max;
                            let color = (230.0 * (1.0 - percent)) as f32 / 255.;
                            Color::from_rgb(1., color, color)
                        } else {
                            let percent = (energy / min).abs();
                            let color = (230.0 * (1.0 - percent)) as f32 / 255.;
                            Color::from_rgb(color, color, 1.)
                        }
                    })
                    .collect()
            }
        }
    }
}

pub struct LatticeViewState {
    scale: f32,
    quadtree: QuadTree,
    center: Point,
    fill_mode: ArrowFillMode,
    grab_state: Option<GrabState>,
}

pub struct GrabState {
    start_mouse_position: Point,
    start_position: Point,
}

impl LatticeViewState {
    pub fn new(system: Option<&System>) -> Self {
        let rectangles: Vec<_> = if let Some(system) = system {
            system.elements()
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    (ItemId(i), Item::Rect(
                        quadtree_f32::Rect {
                            max_x: e.pos.x.0 as f32 + 44.0,
                            max_y: e.pos.y.0 as f32 + 44.0,
                            min_x: e.pos.x.0 as f32 - 44.0,
                            min_y: e.pos.y.0 as f32 - 44.0,
                        }
                    ))
                })
                .collect()
        } else {
            std::iter::empty().collect()
        };

        let quadtree = QuadTree::new(rectangles.into_iter());
        let scale = 0.1;
        let center = Point::new(
            quadtree.bbox().get_center().x,
            quadtree.bbox().get_center().y,
        );

        Self {
            scale,
            quadtree,
            center,
            fill_mode: ArrowFillMode::Default,
            grab_state: None,
        }
    }

    pub fn reload(&mut self, system: &System) {
        let rectangles: Vec<_> = system.elements()
            .iter()
            .enumerate()
            .map(|(i, e)| {
                (ItemId(i), Item::Rect(
                    quadtree_f32::Rect {
                        max_x: e.pos.x.0 as f32 + 44.0,
                        max_y: e.pos.y.0 as f32 + 44.0,
                        min_x: e.pos.x.0 as f32 - 44.0,
                        min_y: e.pos.y.0 as f32 - 44.0,
                    }
                ))
            })
            .collect();

        self.quadtree = QuadTree::new(rectangles.into_iter());
    }
}

pub struct LatticeView<'a> {
    system: &'a System,
    state: &'a mut LatticeViewState,
}

impl<'a> LatticeView<'a> {
    pub fn new(system: &'a System, state: &'a mut LatticeViewState) -> Self {
        Self { system, state }
    }

    pub fn view(mut self) -> Element<'a, AppMessage> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn get_translate(&self, bounds: &Rectangle) -> Vector {
        let Size { width, height } = bounds.size();
        let (hw, hh) = (width / 2. * self.state.scale, height / 2. * self.state.scale);

        Vector::new(
            self.state.center.x - hw,
            self.state.center.y - hh,
        )
    }

    pub fn mouse_to_world(&self, position: Point, bounds: &Rectangle) -> Point {
        let translate = self.get_translate(bounds);
        let x = position.x * (1. / self.state.scale);
        let y = position.y * (1. / self.state.scale);

        Point::new(x, y) - translate * (1. / self.state.scale)
    }
}

impl<'a> Program<AppMessage> for LatticeView<'a> {
    fn update(&mut self, event: Event, bounds: Rectangle, cursor: Cursor) -> (Status, Option<AppMessage>) {
        if let Event::Mouse(event) = event {
            if cursor.position_in(&bounds).is_none() {
                self.state.grab_state = None;
                return (Status::Ignored, None);
            }

            if let Some(mouse_position) = cursor.position() {
                let position = self.mouse_to_world(mouse_position, &bounds);
                let element_id = self.state.quadtree.get_ids_that_overlap(&quadtree_f32::Rect {
                    max_x: position.x + 2.0,
                    max_y: position.y + 2.0,
                    min_x: position.x - 2.0,
                    min_y: position.y - 2.0,
                }).into_iter().next();

                if let iced::mouse::Event::ButtonPressed(button) = &event {
                    if *button == iced::mouse::Button::Left && element_id.is_none() {
                        self.state.grab_state = Some(GrabState {
                            start_position: position,
                            start_mouse_position: mouse_position,
                        });
                    }
                }

                if let iced::mouse::Event::ButtonReleased(button) = &event {
                    if *button == iced::mouse::Button::Left {
                        if self.state.grab_state.is_none() {
                            if let Some(element_id) = element_id {
                                return (Status::Captured, Some(AppMessage::ReverseSpin(element_id.0)));
                            }
                        } else {
                            self.state.grab_state = None;
                        }
                    }
                }

                if let iced::mouse::Event::WheelScrolled { delta } = &event {
                    match delta {
                        ScrollDelta::Lines { y, .. } => {
                            self.state.scale += y * 0.01;
                        }
                        ScrollDelta::Pixels { y, .. } => {
                            self.state.scale += y;
                        }
                    }
                    self.state.scale = self.state.scale.max(0.01);

                    return (Status::Captured, None);
                }

                if let Some(ref grab_state) = self.state.grab_state {
                    let diff = position - grab_state.start_position;
                    self.state.center = self.state.center + diff * 0.1;
                }
            }
        }
        (Status::Ignored, None)
    }


    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(bounds.size());

        let translate = self.get_translate(&bounds);

        frame.fill_rectangle(
            Point::new(0., 0.),
            bounds.size(),
            Color::from_rgb(0.5, 0.5, 0.5),
        );

        frame.translate(translate);
        frame.scale(self.state.scale);

        let l = 200.;
        let h = 50.;
        let al = 80.;
        let ah = 120.;

        let arrow = Path::new(|builder| {
            builder.move_to(Point::new(-l / 2., -h / 2.));
            builder.line_to(Point::new(l / 2., -h / 2.));
            builder.line_to(Point::new(l / 2., -ah / 2.));
            builder.line_to(Point::new(l / 2. + al, 0.));
            builder.line_to(Point::new(l / 2., ah / 2.));
            builder.line_to(Point::new(l / 2., h / 2.));
            builder.line_to(Point::new(-l / 2., h / 2.));
            builder.line_to(Point::new(-l / 2., -h / 2.));

            builder.move_to(Point::new(0., 0.));
            builder.circle(
                Point::new(0., 0.),
                ah / 2.5,
            );
        });

        let colors = self.state.fill_mode.get_colors(self.system);

        for (i, (element, color)) in self.system.elements().iter().zip(colors.into_iter()).enumerate() {
            frame.with_save(|frame| {
                let mut dir = element.magn.map(|v| v.0 as f32);

                if self.system.system_state()[i] {
                    dir *= -1.;
                }

                let angle = dir.y.atan2(dir.x);
                let angle = if angle < 0. {
                    angle + 2. * PI
                } else {
                    angle
                };

                frame.translate(
                    Vector::new(element.pos.x.0 as f32, element.pos.y.0 as f32)
                );
                frame.rotate(angle);

                frame.fill(
                    &arrow,
                    color,
                )
            })
        };

        vec![frame.into_geometry()]
    }
}