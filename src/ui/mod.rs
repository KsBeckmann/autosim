// As coordenadas de desenho derivam de contagens pequenas (número de estados,
// caracteres da entrada), então a conversão de `usize` para `f32` nunca perde
// precisão na prática.
#![allow(clippy::cast_precision_loss)]

use std::collections::HashMap;
use std::f32::consts::PI;
use std::time::Duration;

use iced::mouse;
use iced::time;
use iced::widget::{Canvas, button, canvas, column, container, pick_list, row, text};
use iced::{
    Color, Element, Fill, Point, Rectangle, Renderer, Size, Subscription, Theme, Vector,
};

use crate::parser::{Automaton, AutomatonKind, Program, Symbol};
use crate::runtime::{LastStep, NextStep, Simulator, Status, Verdict};

const TICK_MS: u64 = 800;
const MIN_ZOOM: f32 = 0.2;
const MAX_ZOOM: f32 = 4.0;

/// Câmera do diagrama: deslocamento (pan) e ampliação (zoom) aplicados
/// apenas ao desenho do autômato, permitindo navegar diagramas maiores
/// que a janela arrastando com o mouse e rolando a rodinha.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pan: Vector,
    zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pan: Vector::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimChoice {
    pub index: usize,
    pub label: String,
}

impl std::fmt::Display for SimChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

pub struct State {
    program: Program,
    choices: Vec<SimChoice>,
    selected: SimChoice,
    simulator: Simulator,
    auto: bool,
    camera: Camera,
}

impl State {
    /// Constrói o estado da interface para o programa fornecido.
    ///
    /// # Panics
    ///
    /// Causa panic se o programa não contém nenhuma simulação.
    #[must_use]
    pub fn new(program: Program) -> Self {
        let choices: Vec<SimChoice> = program
            .simulations
            .iter()
            .enumerate()
            .map(|(i, s)| SimChoice {
                index: i,
                label: format!("{} com \"{}\"", s.automaton.value, s.input),
            })
            .collect();

        let selected = choices
            .first()
            .cloned()
            .expect("ao menos uma simulação é obrigatória");
        let simulator = make_simulator(&program, selected.index);

        Self {
            program,
            choices,
            selected,
            simulator,
            auto: false,
            camera: Camera::default(),
        }
    }
}

fn make_simulator(program: &Program, index: usize) -> Simulator {
    let sim_decl = &program.simulations[index];
    let automaton = program
        .automata
        .iter()
        .find(|a| a.name.value == sim_decl.automaton.value)
        .expect("sema garante que o autômato existe")
        .clone();
    Simulator::new(automaton, &sim_decl.input)
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectSimulation(SimChoice),
    StepForward,
    StepBack,
    Reset,
    ToggleAuto,
    Tick,
    /// Arrasto do mouse sobre o canvas: desloca a câmera.
    Pan(Vector),
    /// Rolagem da rodinha: amplia/reduz ao redor do cursor.
    Zoom { factor: f32, anchor: Point },
    /// Restaura a câmera para a posição inicial.
    ResetView,
}

pub fn update(state: &mut State, message: Message) {
    match message {
        Message::SelectSimulation(choice) => {
            state.simulator = make_simulator(&state.program, choice.index);
            state.selected = choice;
            state.auto = false;
            state.camera = Camera::default();
        }
        Message::StepForward => {
            state.simulator.step_forward();
        }
        Message::StepBack => {
            state.simulator.step_back();
            state.auto = false;
        }
        Message::Reset => {
            state.simulator.reset();
            state.auto = false;
        }
        Message::ToggleAuto => {
            if !state.auto && matches!(state.simulator.status(), Status::Done(_)) {
                state.simulator.reset();
            }
            state.auto = !state.auto;
        }
        Message::Tick => {
            if state.auto && !state.simulator.step_forward() {
                state.auto = false;
            }
        }
        Message::Pan(delta) => {
            state.camera.pan += delta;
        }
        Message::Zoom { factor, anchor } => {
            let old_zoom = state.camera.zoom;
            let new_zoom = (old_zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
            // Mantém o ponto do mundo sob o cursor fixo na tela:
            // screen = pan + zoom * world  =>  pan' = anchor - (zoom'/zoom) * (anchor - pan)
            let ratio = new_zoom / old_zoom;
            state.camera.pan = Vector::new(
                ratio.mul_add(-(anchor.x - state.camera.pan.x), anchor.x),
                ratio.mul_add(-(anchor.y - state.camera.pan.y), anchor.y),
            );
            state.camera.zoom = new_zoom;
        }
        Message::ResetView => {
            state.camera = Camera::default();
        }
    }
}

pub fn subscription(state: &State) -> Subscription<Message> {
    if state.auto {
        time::every(Duration::from_millis(TICK_MS)).map(|_| Message::Tick)
    } else {
        Subscription::none()
    }
}

#[must_use]
pub fn view(state: &State) -> Element<'_, Message> {
    let canvas_view: Canvas<&State, Message, Theme, Renderer> =
        canvas(state).width(Fill).height(Fill);

    let header = build_header(state);
    let controls = build_controls(state);

    let content = column![header, canvas_view, controls].spacing(0);

    container(content).width(Fill).height(Fill).into()
}

fn build_header(state: &State) -> Element<'_, Message> {
    let automaton = state.simulator.automaton();
    let kind = match automaton.kind {
        AutomatonKind::Dfa => "AFD",
        AutomatonKind::Nfa => "AFN",
    };

    let title = text(format!("{} ({})", automaton.name.value, kind)).size(18);

    let picker = pick_list(
        state.choices.as_slice(),
        Some(state.selected.clone()),
        Message::SelectSimulation,
    );

    row![
        title,
        text("   |   Simulação:").size(14),
        picker,
    ]
    .spacing(10)
    .padding(10)
    .align_y(iced::Alignment::Center)
    .into()
}

fn build_controls(state: &State) -> Element<'_, Message> {
    let status_label = match state.simulator.status() {
        Status::Running => "Pausado".to_string(),
        Status::Done(Verdict::Accepted) => "ACEITA".to_string(),
        Status::Done(Verdict::Rejected) => "REJEITA".to_string(),
    };
    let status_label = if state.auto {
        "Rodando...".to_string()
    } else {
        status_label
    };

    let play_label = if state.auto { "Pausar" } else { "Rodar" };

    let can_back = state.simulator.step_count() > 0;
    let can_forward = matches!(state.simulator.status(), Status::Running);

    let next_label = match state.simulator.next_step() {
        NextStep::Closure => "ε".to_string(),
        NextStep::Char => {
            let idx = state.simulator.config().consumed;
            let c = state.simulator.input()[idx];
            format!("'{c}'")
        }
        NextStep::Done => "—".to_string(),
    };

    let back = if can_back {
        button("← Voltar").on_press(Message::StepBack)
    } else {
        button("← Voltar")
    };
    let step = if can_forward {
        button("Passo →").on_press(Message::StepForward)
    } else {
        button("Passo →")
    };
    let play = button(play_label).on_press(Message::ToggleAuto);
    let reset = button("Reset").on_press(Message::Reset);
    let center = button("Centralizar").on_press(Message::ResetView);

    let info = text(format!(
        "  Próximo: {}  |  Lidos: {}/{}  |  {}  |  Zoom: {:.0}%",
        next_label,
        state.simulator.config().consumed,
        state.simulator.input().len(),
        status_label,
        state.camera.zoom * 100.0
    ))
    .size(14);

    row![back, step, play, reset, center, info]
        .spacing(10)
        .padding(10)
        .align_y(iced::Alignment::Center)
        .into()
}

/// Estado interno do canvas: posição do cursor durante um arrasto.
#[derive(Debug, Default)]
pub struct Interaction {
    drag: Option<Point>,
}

impl canvas::Program<Message> for &State {
    type State = Interaction;

    fn update(
        &self,
        interaction: &mut Interaction,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let canvas::Event::Mouse(mouse_event) = event else {
            return None;
        };

        match mouse_event {
            mouse::Event::ButtonPressed(mouse::Button::Left) => {
                let pos = cursor.position_over(bounds)?;
                interaction.drag = Some(pos);
                Some(canvas::Action::capture())
            }
            mouse::Event::CursorMoved { position } => {
                let last = interaction.drag?;
                interaction.drag = Some(*position);
                let delta = *position - last;
                Some(canvas::Action::publish(Message::Pan(delta)).and_capture())
            }
            mouse::Event::ButtonReleased(mouse::Button::Left) => {
                interaction.drag.take()?;
                Some(canvas::Action::capture())
            }
            mouse::Event::WheelScrolled { delta } => {
                let pos = cursor.position_in(bounds)?;
                let amount = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / 60.0,
                };
                let factor = 1.1_f32.powf(amount);
                Some(
                    canvas::Action::publish(Message::Zoom {
                        factor,
                        anchor: pos,
                    })
                    .and_capture(),
                )
            }
            _ => None,
        }
    }

    fn mouse_interaction(
        &self,
        interaction: &Interaction,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if interaction.drag.is_some() {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        _interaction: &Interaction,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let h = bounds.height;

        let simulator = &self.simulator;
        let bg_color = match simulator.status() {
            Status::Done(Verdict::Rejected) => Color::from_rgb(0.20, 0.07, 0.09),
            Status::Done(Verdict::Accepted) => Color::from_rgb(0.07, 0.18, 0.10),
            Status::Running => Color::from_rgb(0.12, 0.12, 0.15),
        };

        frame.fill_rectangle(Point::ORIGIN, Size::new(w, h), bg_color);

        let automaton = simulator.automaton();
        let state_radius = 40.0;
        let positions = layout_states(automaton, w, h);
        let indices = state_indices(automaton);

        let active = active_transitions(simulator, automaton);

        // O diagrama é desenhado sob a câmera (pan + zoom); a entrada e o
        // histórico ficam fixos na tela, como um HUD.
        frame.with_save(|f| {
            f.translate(self.camera.pan);
            f.scale(self.camera.zoom);
            draw_transitions(f, automaton, &positions, &indices, state_radius, &active);
            draw_states(f, simulator, automaton, &positions, state_radius, bg_color);
            draw_initial_arrow(f, &positions, &indices, automaton, state_radius);
        });

        draw_input_string(&mut frame, simulator, w, h);
        draw_history(&mut frame, simulator, h);

        vec![frame.into_geometry()]
    }
}

fn state_indices(automaton: &Automaton) -> HashMap<String, usize> {
    automaton
        .states
        .iter()
        .enumerate()
        .map(|(i, s)| (s.value.clone(), i))
        .collect()
}

/// Distância mínima entre centros de estados vizinhos no círculo, deixando
/// folga para os rótulos e as setas curvas entre as "bolinhas" (raio 40).
const MIN_NEIGHBOR_GAP: f32 = 160.0;

fn layout_states(automaton: &Automaton, w: f32, h: f32) -> Vec<Point> {
    let n = automaton.states.len();
    let cx = w / 2.0;
    let cy = h / 2.0 - 30.0;
    let usable = w.min(h - 200.0);
    let fit_radius = (usable * 0.32).max(90.0);

    match n {
        0 => vec![],
        1 => vec![Point::new(cx, cy)],
        2 => vec![
            Point::new(cx - fit_radius, cy),
            Point::new(cx + fit_radius, cy),
        ],
        _ => {
            // A corda entre vizinhos em um círculo de raio R é 2·R·sin(π/n);
            // para muitos estados o raio cresce além da janela e o diagrama
            // fica navegável com o mouse (pan/zoom da câmera).
            let needed = MIN_NEIGHBOR_GAP / (2.0 * (PI / n as f32).sin());
            let radius = fit_radius.max(needed);
            (0..n)
                .map(|i| {
                    let angle = (i as f32) * 2.0 * PI / (n as f32) - PI / 2.0;
                    Point::new(cx + radius * angle.cos(), cy + radius * angle.sin())
                })
                .collect()
        }
    }
}

fn active_transitions(simulator: &Simulator, automaton: &Automaton) -> Vec<usize> {
    let current = simulator.config();
    let Some(prev) = simulator.prev_config() else {
        return Vec::new();
    };
    let mut active = Vec::new();

    match current.last_step {
        LastStep::None => {}
        LastStep::Closure => {
            for (i, tr) in automaton.transitions.iter().enumerate() {
                if matches!(tr.symbol.value, Symbol::Epsilon)
                    && prev.current.contains(&tr.from.value)
                    && current.current.contains(&tr.to.value)
                    && !prev.current.contains(&tr.to.value)
                {
                    active.push(i);
                }
            }
        }
        LastStep::Char(c) => {
            for (i, tr) in automaton.transitions.iter().enumerate() {
                if let Symbol::Char(sym) = tr.symbol.value
                    && sym == c
                    && prev.current.contains(&tr.from.value)
                    && current.current.contains(&tr.to.value)
                {
                    active.push(i);
                }
            }
        }
    }

    active
}

fn draw_transitions(
    frame: &mut canvas::Frame,
    automaton: &Automaton,
    positions: &[Point],
    indices: &HashMap<String, usize>,
    state_radius: f32,
    active: &[usize],
) {
    let normal_color = Color::from_rgb(0.5, 0.5, 0.55);
    let active_color = Color::from_rgb(0.4, 0.7, 1.0);

    let mut grouped: HashMap<(usize, usize), Vec<(String, bool)>> = HashMap::new();
    for (i, tr) in automaton.transitions.iter().enumerate() {
        let from = indices[&tr.from.value];
        let to = indices[&tr.to.value];
        let label = match &tr.symbol.value {
            Symbol::Char(c) => format!("'{c}'"),
            Symbol::Epsilon => "ε".to_string(),
        };
        let is_active = active.contains(&i);
        grouped.entry((from, to)).or_default().push((label, is_active));
    }

    for ((from, to), entries) in &grouped {
        let any_active = entries.iter().any(|(_, a)| *a);
        let labels: Vec<&str> = entries.iter().map(|(s, _)| s.as_str()).collect();
        let label = labels.join(",");
        let style = EdgeStyle {
            color: if any_active { active_color } else { normal_color },
            width: if any_active { 3.0 } else { 1.5 },
            label_color: if any_active {
                Color::WHITE
            } else {
                Color::from_rgb(0.7, 0.7, 0.7)
            },
        };

        if from == to {
            draw_self_loop(frame, positions[*from], state_radius, &label, style);
        } else {
            let has_reverse = grouped.contains_key(&(*to, *from));
            let curve_side: f32 = if has_reverse {
                if from < to { 1.0 } else { -1.0 }
            } else {
                1.0
            };
            draw_curved_arrow(
                frame,
                positions[*from],
                positions[*to],
                state_radius,
                &label,
                style,
                curve_side,
            );
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct EdgeStyle {
    color: Color,
    width: f32,
    label_color: Color,
}

fn draw_curved_arrow(
    frame: &mut canvas::Frame,
    from: Point,
    to: Point,
    state_radius: f32,
    label: &str,
    style: EdgeStyle,
    side: f32,
) {
    let EdgeStyle {
        color,
        width,
        label_color,
    } = style;
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let dist = dx.hypot(dy).max(1.0);
    let nx = dx / dist;
    let ny = dy / dist;

    let start = Point::new(from.x + nx * state_radius, from.y + ny * state_radius);
    let end = Point::new(to.x - nx * state_radius, to.y - ny * state_radius);

    let mx = f32::midpoint(start.x, end.x);
    let my = f32::midpoint(start.y, end.y);
    let curve_offset = 35.0 * side;
    let perp_x = -ny * curve_offset;
    let perp_y = nx * curve_offset;
    let ctrl = Point::new(mx + perp_x, my + perp_y);

    let path = canvas::Path::new(|p| {
        p.move_to(start);
        p.quadratic_curve_to(ctrl, end);
    });
    frame.stroke(
        &path,
        canvas::Stroke::default().with_width(width).with_color(color),
    );

    let angle = (end.y - ctrl.y).atan2(end.x - ctrl.x);
    let arrow_size = 10.0;
    let arrow = canvas::Path::new(|p| {
        p.move_to(end);
        p.line_to(Point::new(
            end.x - arrow_size * (angle - 0.4).cos(),
            end.y - arrow_size * (angle - 0.4).sin(),
        ));
        p.move_to(end);
        p.line_to(Point::new(
            end.x - arrow_size * (angle + 0.4).cos(),
            end.y - arrow_size * (angle + 0.4).sin(),
        ));
    });
    frame.stroke(
        &arrow,
        canvas::Stroke::default().with_width(width).with_color(color),
    );

    let label_y_offset = if side > 0.0 { -5.0 } else { 15.0 };
    let label_text = canvas::Text {
        content: label.to_string(),
        position: Point::new(ctrl.x, ctrl.y + label_y_offset),
        color: label_color,
        size: iced::Pixels(14.0),
        align_x: iced::alignment::Horizontal::Center.into(),
        ..canvas::Text::default()
    };
    frame.fill_text(label_text);
}

fn draw_self_loop(
    frame: &mut canvas::Frame,
    pos: Point,
    state_radius: f32,
    label: &str,
    style: EdgeStyle,
) {
    let EdgeStyle {
        color,
        width,
        label_color,
    } = style;
    let loop_h = 45.0;
    let dir = -1.0;

    let start = Point::new(pos.x - 15.0, pos.y + dir * state_radius);
    let end = Point::new(pos.x + 15.0, pos.y + dir * state_radius);
    let ctrl1 = Point::new(pos.x - 30.0, pos.y + dir * (state_radius + loop_h));
    let ctrl2 = Point::new(pos.x + 30.0, pos.y + dir * (state_radius + loop_h));

    let path = canvas::Path::new(|p| {
        p.move_to(start);
        p.bezier_curve_to(ctrl1, ctrl2, end);
    });
    frame.stroke(
        &path,
        canvas::Stroke::default().with_width(width).with_color(color),
    );

    let angle = (end.y - ctrl2.y).atan2(end.x - ctrl2.x);
    let arrow_size = 8.0;
    let arrow = canvas::Path::new(|p| {
        p.move_to(end);
        p.line_to(Point::new(
            end.x - arrow_size * (angle - 0.4).cos(),
            end.y - arrow_size * (angle - 0.4).sin(),
        ));
        p.move_to(end);
        p.line_to(Point::new(
            end.x - arrow_size * (angle + 0.4).cos(),
            end.y - arrow_size * (angle + 0.4).sin(),
        ));
    });
    frame.stroke(
        &arrow,
        canvas::Stroke::default().with_width(width).with_color(color),
    );

    let label_text = canvas::Text {
        content: label.to_string(),
        position: Point::new(pos.x, pos.y + dir * (state_radius + loop_h + 5.0)),
        color: label_color,
        size: iced::Pixels(14.0),
        align_x: iced::alignment::Horizontal::Center.into(),
        ..canvas::Text::default()
    };
    frame.fill_text(label_text);
}

fn draw_states(
    frame: &mut canvas::Frame,
    simulator: &Simulator,
    automaton: &Automaton,
    positions: &[Point],
    state_radius: f32,
    bg_color: Color,
) {
    let current = &simulator.config().current;
    let status = simulator.status();
    let finished = matches!(status, Status::Done(_));

    for (i, st) in automaton.states.iter().enumerate() {
        let pos = positions[i];
        let is_current = current.contains(&st.value);
        let is_final = automaton.finals.iter().any(|f| f.value == st.value);

        let color = match &status {
            Status::Done(Verdict::Rejected) => Color::from_rgb(0.9, 0.2, 0.2),
            Status::Done(Verdict::Accepted) if is_current => Color::from_rgb(0.2, 0.8, 0.3),
            Status::Running if is_current => Color::from_rgb(0.3, 0.6, 1.0),
            _ => Color::from_rgb(0.3, 0.3, 0.35),
        };

        let outer = canvas::Path::circle(pos, state_radius);
        frame.fill(&outer, color);

        let inner = canvas::Path::circle(pos, state_radius - 3.0);
        frame.fill(&inner, bg_color);

        if is_final {
            let dbl = canvas::Path::circle(pos, state_radius - 8.0);
            frame.fill(&dbl, color);
            let dbl_inner = canvas::Path::circle(pos, state_radius - 11.0);
            frame.fill(&dbl_inner, bg_color);
        }

        let label = canvas::Text {
            content: st.value.clone(),
            position: Point::new(pos.x, pos.y - 8.0),
            color: Color::WHITE,
            size: iced::Pixels(18.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            ..canvas::Text::default()
        };
        frame.fill_text(label);

        if is_current && !finished {
            let pulse = canvas::Path::circle(pos, state_radius + 5.0);
            frame.stroke(
                &pulse,
                canvas::Stroke::default()
                    .with_width(2.0)
                    .with_color(Color::from_rgba(0.3, 0.6, 1.0, 0.5)),
            );
        }
    }
}

fn draw_initial_arrow(
    frame: &mut canvas::Frame,
    positions: &[Point],
    indices: &HashMap<String, usize>,
    automaton: &Automaton,
    state_radius: f32,
) {
    let Some(&idx) = indices.get(&automaton.initial.value) else {
        return;
    };
    let pos = positions[idx];

    let start = Point::new(pos.x - state_radius - 40.0, pos.y);
    let end = Point::new(pos.x - state_radius - 2.0, pos.y);
    let path = canvas::Path::new(|p| {
        p.move_to(start);
        p.line_to(end);
        p.move_to(end);
        p.line_to(Point::new(end.x - 8.0, end.y - 5.0));
        p.move_to(end);
        p.line_to(Point::new(end.x - 8.0, end.y + 5.0));
    });
    frame.stroke(
        &path,
        canvas::Stroke::default()
            .with_width(2.0)
            .with_color(Color::WHITE),
    );
}

fn draw_input_string(frame: &mut canvas::Frame, simulator: &Simulator, w: f32, h: f32) {
    let chars = simulator.input();
    let cfg = simulator.config();
    let consumed = cfg.consumed;
    let finished = matches!(simulator.status(), Status::Done(_));
    let rejected_at = if cfg.current.is_empty() && consumed > 0 {
        Some(consumed - 1)
    } else {
        None
    };

    let cell_w = 25.0;
    let total_w = (chars.len().max(1) as f32) * cell_w;
    let start_x = (w - total_w) / 2.0;
    let y = h - 100.0;

    let title = canvas::Text {
        content: "Entrada:".to_string(),
        position: Point::new(start_x - 80.0, y),
        color: Color::from_rgb(0.7, 0.7, 0.7),
        size: iced::Pixels(16.0),
        ..canvas::Text::default()
    };
    frame.fill_text(title);

    if chars.is_empty() {
        let empty = canvas::Text {
            content: "ε".to_string(),
            position: Point::new(start_x + 11.0, y),
            color: Color::from_rgb(0.7, 0.7, 0.7),
            size: iced::Pixels(18.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            ..canvas::Text::default()
        };
        frame.fill_text(empty);
        return;
    }

    for (i, c) in chars.iter().enumerate() {
        let x = (i as f32).mul_add(cell_w, start_x);
        let (bg, fg) = if Some(i) == rejected_at {
            (
                Color::from_rgb(0.55, 0.15, 0.15),
                Color::from_rgb(1.0, 0.75, 0.75),
            )
        } else if i < consumed {
            (
                Color::from_rgb(0.2, 0.4, 0.2),
                Color::from_rgb(0.6, 0.9, 0.6),
            )
        } else if i == consumed && !finished {
            (Color::from_rgb(0.3, 0.5, 0.8), Color::WHITE)
        } else {
            (
                Color::from_rgb(0.25, 0.25, 0.28),
                Color::from_rgb(0.6, 0.6, 0.6),
            )
        };

        frame.fill_rectangle(Point::new(x, y - 2.0), Size::new(22.0, 26.0), bg);

        let char_text = canvas::Text {
            content: c.to_string(),
            position: Point::new(x + 11.0, y),
            color: fg,
            size: iced::Pixels(18.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            ..canvas::Text::default()
        };
        frame.fill_text(char_text);
    }
}

fn draw_history(frame: &mut canvas::Frame, simulator: &Simulator, h: f32) {
    let history = simulator.history();
    if history.len() <= 1 {
        return;
    }
    let y = h - 55.0;

    let title = canvas::Text {
        content: "Histórico:".to_string(),
        position: Point::new(20.0, y),
        color: Color::from_rgb(0.7, 0.7, 0.7),
        size: iced::Pixels(14.0),
        ..canvas::Text::default()
    };
    frame.fill_text(title);

    for i in 1..history.len() {
        let prev = &history[i - 1];
        let cur = &history[i];
        let label = match cur.last_step {
            LastStep::Closure => {
                format!("ε: {}>{}", format_set(&prev.current), format_set(&cur.current))
            }
            LastStep::Char(c) => {
                format!("'{}': {}>{}", c, format_set(&prev.current), format_set(&cur.current))
            }
            LastStep::None => "—".to_string(),
        };

        let entry = canvas::Text {
            content: label,
            position: Point::new(((i - 1) as f32).mul_add(140.0, 110.0), y),
            color: Color::from_rgb(0.8, 0.8, 0.8),
            size: iced::Pixels(13.0),
            ..canvas::Text::default()
        };
        frame.fill_text(entry);
    }
}

fn format_set(set: &std::collections::BTreeSet<String>) -> String {
    if set.len() == 1 {
        set.iter().next().cloned().unwrap_or_default()
    } else if set.is_empty() {
        "∅".to_string()
    } else {
        let xs: Vec<&str> = set.iter().map(String::as_str).collect();
        format!("{{{}}}", xs.join(","))
    }
}

