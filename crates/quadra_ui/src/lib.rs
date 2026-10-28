// Quadra's main window: the mockup A layout. Panels hold real controls where a
// later M1 step will wire them up; nothing here edits a scene yet.

pub mod theme;

use egui::{Align, Align2, Color32, FontId, Layout, Pos2, Rect, Sense, Stroke, Ui, Vec2, pos2, vec2};
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use theme::*;

pub const NOTICE: &str = "Provided \"as is\", without warranty, under the Apache 2.0 licence. \
To the extent permitted by law, the authors are not liable for any loss or damage from using it. \
Back up your work.";

#[derive(Clone, Copy, PartialEq)]
pub enum Tool {
    Select,
    Move,
    Rotate,
    Scale,
}

#[derive(Clone, Copy, PartialEq, Hash)]
enum Tab {
    Explorer,
    Viewports,
    CommandPanel,
}

const VIEW_NAMES: [&str; 4] = ["Top", "Front", "Left", "Perspective"];
const CMD_TABS: [&str; 6] = ["Create", "Modify", "Hierarchy", "Motion", "Display", "Utilities"];
const CREATE_KINDS: [&str; 7] = ["Geometry", "Shapes", "Lights", "Cameras", "Helpers", "Space Warps", "Systems"];
const PRIMITIVES: [&str; 11] = ["Box", "Cone", "Sphere", "GeoSphere", "Cylinder", "Tube", "Torus", "Pyramid", "Teapot", "Plane", "TextPlus"];

// The panel state the dock tabs need while the dock itself is borrowed.
struct Panels {
    active_view: usize,
    cmd_tab: usize,
    create_kind: usize,
    object_name: String,
    explorer_filter: String,
}

pub struct MainWindow {
    dock: DockState<Tab>,
    panels: Panels,
    tool: Tool,
    about_open: bool,
}

impl MainWindow {
    pub fn new(ctx: &egui::Context) -> Self {
        theme::apply(ctx);
        // Explorer left, viewports centre, command panel right, sized as in the mockup.
        let mut dock = DockState::new(vec![Tab::Viewports]);
        let tree = dock.main_surface_mut();
        let [centre, _] = tree.split_right(NodeIndex::root(), 0.88, vec![Tab::CommandPanel]);
        tree.split_left(centre, 0.19, vec![Tab::Explorer]);
        MainWindow {
            dock,
            panels: Panels { active_view: 3, cmd_tab: 0, create_kind: 0, object_name: String::new(), explorer_filter: String::new() },
            tool: Tool::Select,
            about_open: false,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let frame = |fill| egui::Frame::new().fill(fill);
        egui::Panel::top("menu").exact_size(27.0).frame(frame(UI).inner_margin(egui::Margin::symmetric(6, 0))).show(ui, |ui| self.menu_bar(ui));
        egui::Panel::top("toolbar").exact_size(39.0).frame(frame(UI).inner_margin(egui::Margin::symmetric(4, 0))).show(ui, |ui| self.toolbar(ui));
        egui::Panel::bottom("status").exact_size(66.0).frame(frame(UI)).show(ui, status_bar);
        egui::Panel::bottom("track").exact_size(33.0).frame(frame(UI)).show(ui, track_bar);
        egui::Panel::bottom("slider").exact_size(24.0).frame(frame(UI)).show(ui, time_slider);
        egui::Panel::left("layouts").exact_size(56.0).resizable(false).frame(frame(STRIP)).show(ui, layout_strip);
        egui::CentralPanel::default().frame(frame(UI)).show(ui, |ui| {
            let mut style = egui_dock::Style::from_egui(ui.style());
            style.tab_bar.bg_fill = UI;
            style.tab_bar.height = 22.0;
            style.tab.tab_body.bg_fill = UI;
            style.separator.color_idle = UI;
            style.separator.width = 4.0;
            DockArea::new(&mut self.dock)
                .style(style)
                .show_close_buttons(false)
                .show_add_buttons(false)
                .show_leaf_close_all_buttons(false)
                .show_leaf_collapse_buttons(false)
                .show_inside(ui, &mut self.panels);
        });

        if self.about_open {
            egui::Window::new("About Quadra")
                .collapsible(false)
                .resizable(false)
                .open(&mut self.about_open)
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ui.ctx(), |ui| {
                    ui.set_max_width(420.0);
                    ui.label(format!("Quadra {}", env!("CARGO_PKG_VERSION")));
                    ui.label("Free, open-source 3D modelling, animation and rendering.");
                    ui.add_space(8.0);
                    ui.label(NOTICE);
                    ui.add_space(8.0);
                    ui.label("Not affiliated with or endorsed by Autodesk. 3ds Max is a trademark of Autodesk, Inc.");
                });
        }
    }

    fn menu_bar(&mut self, ui: &mut Ui) {
        // Menus that have nothing yet say which milestone fills them.
        const LATER: [(&str, &str); 12] = [
            ("Tools", "M1"),
            ("Group", "M2"),
            ("Views", "M1"),
            ("Create", "M1"),
            ("Modifiers", "M2"),
            ("Animation", "M4"),
            ("Graph Editors", "M4"),
            ("Rendering", "M3"),
            ("Civil View", "not planned"),
            ("Customize", "M1"),
            ("Scripting", "M5"),
            ("Content", "not planned"),
        ];
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                for item in ["New", "Open...", "Save", "Save As...", "Import...", "Export..."] {
                    ui.add_enabled(false, egui::Button::new(item));
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button("Edit", |ui| {
                for item in ["Undo", "Redo", "Delete", "Clone", "Select All", "Select None"] {
                    ui.add_enabled(false, egui::Button::new(item));
                }
            });
            for (name, when) in LATER {
                ui.menu_button(name, |ui| {
                    let text = if when == "not planned" { "Not planned".to_string() } else { format!("Arrives in {when}") };
                    ui.add_enabled(false, egui::Button::new(text));
                });
            }
            ui.menu_button("Help", |ui| {
                if ui.button("About Quadra").clicked() {
                    self.about_open = true;
                }
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                field(ui, "Default", 200.0, true);
                ui.label("Workspaces:");
            });
        });
    }

    fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            icon_button(ui, Icon::Undo, false, "Undo (Ctrl+Z)");
            icon_button(ui, Icon::Redo, false, "Redo (Ctrl+Y)");
            separator(ui);
            icon_button(ui, Icon::Link, false, "Link (Select and Link)");
            icon_button(ui, Icon::Unlink, false, "Unlink (Unlink Selection)");
            separator(ui);
            field(ui, "All", 70.0, true);
            for (tool, icon, tip) in [
                (Tool::Select, Icon::Select, "Select (Select Object) - Q"),
                (Tool::Move, Icon::Move, "Move (Select and Move) - W"),
                (Tool::Rotate, Icon::Rotate, "Rotate (Select and Rotate) - E"),
                (Tool::Scale, Icon::Scale, "Scale (Select and Uniform Scale) - R"),
            ] {
                if icon_button(ui, icon, self.tool == tool, tip).clicked() {
                    self.tool = tool;
                }
            }
            icon_button(ui, Icon::RegionRect, false, "Region: Rectangle (Rectangular Selection Region)");
            separator(ui);
            field(ui, "View", 70.0, true);
            separator(ui);
            icon_button(ui, Icon::Snap, false, "Snap (Snaps Toggle) - S");
            icon_button(ui, Icon::AngleSnap, false, "Angle Snap (Angle Snap Toggle) - A");
            icon_button(ui, Icon::PercentSnap, false, "Percent Snap (Percent Snap Toggle)");
            separator(ui);
            field(ui, "Create Selection Set", 150.0, true);
            separator(ui);
            icon_button(ui, Icon::Mirror, false, "Mirror");
            icon_button(ui, Icon::Align, false, "Align - Alt+A");
            separator(ui);
            icon_button(ui, Icon::Layers, false, "Layers (Toggle Layer Explorer)");
            icon_button(ui, Icon::Explorer, true, "Scene Explorer (Toggle Scene Explorer)");
            separator(ui);
            icon_button(ui, Icon::RenderSetup, false, "Render Setup - F10");
            icon_button(ui, Icon::Render, false, "Render (Render Production) - Shift+Q");
        });
    }
}

impl TabViewer for Panels {
    type Tab = Tab;

    fn id(&mut self, tab: &mut Tab) -> egui::Id {
        egui::Id::new(("dock-tab", *tab as u8))
    }

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        match tab {
            Tab::Explorer => "Scene Explorer",
            Tab::Viewports => "Viewports",
            Tab::CommandPanel => "Command Panel",
        }
        .into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Tab) {
        match tab {
            Tab::Explorer => self.explorer(ui),
            Tab::Viewports => self.viewports(ui),
            Tab::CommandPanel => self.command_panel(ui),
        }
    }

    fn is_closeable(&self, _: &Tab) -> bool {
        false
    }

    fn scroll_bars(&self, tab: &Tab) -> [bool; 2] {
        [false, *tab == Tab::CommandPanel]
    }
}

impl Panels {
    fn explorer(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        egui::MenuBar::new().ui(ui, |ui| {
            for name in ["Select", "Display", "Edit", "Customize"] {
                ui.menu_button(name, |ui| {
                    ui.add_enabled(false, egui::Button::new("Arrives with selection (M1)"));
                });
            }
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.explorer_filter).hint_text("Find").desired_width(150.0));
        });
        ui.add_space(6.0);
        let (head, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
        let p = ui.painter();
        p.rect_filled(head, 0.0, HEADER);
        p.text(head.left_center() + vec2(6.0, 0.0), Align2::LEFT_CENTER, "Name (Sorted Ascending)", FontId::proportional(13.0), INK);
        p.text(head.right_center() - vec2(70.0, 0.0), Align2::LEFT_CENTER, "Frozen", FontId::proportional(13.0), INK);
        ui.add_space(6.0);
        ui.colored_label(DIM, "The scene is empty.");

        // Layer picker sits at the bottom, as in the mockup.
        let bottom = ui.max_rect().bottom();
        let rect = Rect::from_min_size(pos2(ui.max_rect().left() + 4.0, bottom - 30.0), vec2(226.0, 22.0));
        ui.painter().rect_filled(rect, 0.0, FIELD);
        ui.painter().text(rect.left_center() + vec2(6.0, 0.0), Align2::LEFT_CENTER, "Default", FontId::proportional(12.0), LAYER_TEXT);
    }

    fn viewports(&mut self, ui: &mut Ui) {
        let area = ui.available_rect_before_wrap();
        let gap = 4.0;
        let w = (area.width() - gap) / 2.0;
        let h = (area.height() - gap) / 2.0;
        for (i, name) in VIEW_NAMES.iter().enumerate() {
            let min = area.min + vec2((i % 2) as f32 * (w + gap), (i / 2) as f32 * (h + gap));
            let rect = Rect::from_min_size(min, vec2(w, h));
            let resp = ui.interact(rect, ui.id().with(("view", i)), Sense::click());
            if resp.clicked() {
                self.active_view = i;
            }
            let p = ui.painter_at(rect);
            p.rect_filled(rect, 0.0, VIEWPORT);
            let shading = if i == 3 { "Default Shading" } else { "Wireframe" };
            p.text(rect.min + vec2(6.0, 4.0), Align2::LEFT_TOP, format!("[+] [{name}] [Standard] [{shading}]"), FontId::proportional(12.0), Color32::from_gray(0xcd));
            if i == self.active_view {
                p.rect_stroke(rect.shrink(1.0), 0.0, Stroke::new(2.0, ACTIVE_VIEW), egui::StrokeKind::Inside);
            }
        }
        ui.allocate_rect(area, Sense::hover());
    }

    fn command_panel(&mut self, ui: &mut Ui) {
        ui.add_space(6.0);
        // Six panel tabs: Create, Modify, Hierarchy, Motion, Display, Utilities.
        ui.columns(6, |cols| {
            for (i, col) in cols.iter_mut().enumerate() {
                let (rect, resp) = col.allocate_exact_size(vec2(col.available_width(), 36.0), Sense::click());
                col.painter().rect_filled(rect, 0.0, if self.cmd_tab == i { ROW1 } else { TAB });
                draw_icon(col.painter(), rect.shrink(8.0), CMD_ICONS[i]);
                if resp.on_hover_text(CMD_TABS[i]).clicked() {
                    self.cmd_tab = i;
                }
            }
        });
        if self.cmd_tab != 0 {
            ui.add_space(12.0);
            ui.colored_label(DIM, format!("{} panel arrives in a later step.", CMD_TABS[self.cmd_tab]));
            return;
        }
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let size = ui.available_width() / 7.0;
            for (i, kind) in CREATE_KINDS.iter().enumerate() {
                let (rect, resp) = ui.allocate_exact_size(vec2(size, 32.0), Sense::click());
                if self.create_kind == i {
                    ui.painter().rect_filled(rect.shrink2(vec2(2.0, 0.0)), 0.0, ON);
                }
                draw_icon(ui.painter(), Rect::from_center_size(rect.center(), Vec2::splat(18.0)), KIND_ICONS[i]);
                if resp.on_hover_text(*kind).clicked() {
                    self.create_kind = i;
                }
            }
        });
        ui.add_space(4.0);
        field(ui, "Standard Primitives", ui.available_width(), true);
        ui.add_space(8.0);
        rollout(ui, "Object Type", |ui| {
            ui.add_enabled(false, egui::Checkbox::new(&mut false, "AutoGrid"));
            let w = (ui.available_width() - 12.0) / 2.0;
            egui::Grid::new("prims").num_columns(2).spacing(vec2(12.0, 5.0)).show(ui, |ui| {
                for (i, name) in PRIMITIVES.iter().enumerate() {
                    ui.add_sized([w, 21.0], egui::Button::new(*name)).on_hover_text("Creating objects arrives in step 5");
                    if i % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
        });
        rollout(ui, "Name and Color", |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.object_name).desired_width(ui.available_width() - 34.0));
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::hover());
                ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(0xcd, 0x8c, 0x54));
            });
        });
    }
}

fn layout_strip(ui: &mut Ui) {
    // Viewport layout thumbnail: the current 2x2 layout.
    let r = ui.max_rect();
    let thumb = Rect::from_min_size(pos2(r.left() + 9.0, r.bottom() - 90.0), Vec2::splat(38.0));
    let p = ui.painter();
    p.rect_filled(thumb, 0.0, ON);
    let inner = thumb.shrink(4.0);
    let cell = (inner.width() - 2.0) / 2.0;
    for i in 0..4 {
        let min = inner.min + vec2((i % 2) as f32 * (cell + 2.0), (i / 2) as f32 * (cell + 2.0));
        let c = Rect::from_min_size(min, Vec2::splat(cell));
        p.rect_filled(c, 0.0, Color32::from_rgb(0x3d, 0x5f, 0x8d));
        p.rect_stroke(c, 0.0, Stroke::new(1.0, Color32::from_rgb(0x9c, 0xb9, 0xdf)), egui::StrokeKind::Inside);
    }
}

fn time_slider(ui: &mut Ui) {
    let r = ui.max_rect();
    let left = r.left() + 56.0 + r.width() * 0.2;
    let y = r.center().y;
    let p = ui.painter();
    let arrow = |x: f32, text: &str| {
        let b = Rect::from_center_size(pos2(x, y), vec2(22.0, 20.0));
        p.rect_filled(b, 0.0, Color32::from_gray(0x52));
        p.text(b.center(), Align2::CENTER_CENTER, text, FontId::proportional(12.0), Color32::from_gray(0xbb));
    };
    arrow(left, "<");
    let thumb = Rect::from_min_size(pos2(left + 14.0, y - 10.0), vec2(135.0, 20.0));
    p.rect_filled(thumb, 0.0, FIELD2);
    p.text(thumb.center(), Align2::CENTER_CENTER, "0 / 100", FontId::proportional(13.0), INK);
    arrow(thumb.right() + 14.0, ">");
}

fn track_bar(ui: &mut Ui) {
    let r = ui.max_rect();
    let bar = Rect::from_min_max(pos2(r.left() + 78.0, r.top()), pos2(r.right() - 53.0, r.bottom()));
    let p = ui.painter();
    p.rect_filled(bar, 0.0, TRACK);
    let step = (bar.width() - 20.0) / 100.0;
    for f in 0..=100 {
        let x = bar.left() + 10.0 + f as f32 * step;
        let tall = f % 5 == 0;
        p.line_segment([pos2(x, bar.top()), pos2(x, bar.top() + if tall { 9.0 } else { 5.0 })], Stroke::new(1.0, Color32::from_gray(0x6a)));
        if tall {
            p.text(pos2(x, bar.bottom() - 2.0), Align2::CENTER_BOTTOM, f.to_string(), FontId::proportional(12.0), Color32::from_gray(0xcf));
        }
    }
    let cur = Rect::from_min_size(pos2(bar.left() + 10.0 - 4.5, bar.top()), vec2(9.0, bar.height()));
    p.rect_filled(cur, 0.0, KEY_CURSOR);
    p.text(pos2(cur.center().x, bar.bottom() - 2.0), Align2::CENTER_BOTTOM, "0", FontId::proportional(12.0), Color32::from_gray(0xcf));
}

fn status_bar(ui: &mut Ui) {
    let r = ui.max_rect().shrink2(vec2(6.0, 4.0));
    let p = ui.painter();
    // Script listener mini-boxes (filled in by M5).
    let half = (r.height() - 2.0) / 2.0;
    p.rect_filled(Rect::from_min_size(r.min, vec2(122.0, half)), 0.0, Color32::from_rgb(0xdc, 0xc4, 0xc4));
    p.rect_filled(Rect::from_min_size(r.min + vec2(0.0, half + 2.0), vec2(122.0, half)), 0.0, Color32::WHITE);
    let text_x = r.left() + 134.0;
    p.text(pos2(text_x, r.top() + half / 2.0), Align2::LEFT_CENTER, "None Selected", FontId::proportional(13.0), INK);
    p.text(pos2(text_x, r.bottom() - half / 2.0), Align2::LEFT_CENTER, "Click or click-and-drag to select objects", FontId::proportional(13.0), INK);

    let mut x = r.left() + 760.0;
    for axis in ["X:", "Y:", "Z:"] {
        p.text(pos2(x, r.top() + half / 2.0), Align2::LEFT_CENTER, axis, FontId::proportional(13.0), INK);
        p.rect_filled(Rect::from_min_size(pos2(x + 18.0, r.top() + 1.0), vec2(70.0, half - 2.0)), 0.0, FIELD);
        x += 104.0;
    }
    p.text(pos2(x + 8.0, r.top() + half / 2.0), Align2::LEFT_CENTER, "Grid = 10.0", FontId::proportional(13.0), INK);

    // Playback controls and key buttons, right-aligned.
    let right = r.right();
    let mid = r.top() + half / 2.0;
    for (i, icon) in [Icon::GoStart, Icon::PrevFrame, Icon::Play, Icon::NextFrame, Icon::GoEnd].iter().enumerate() {
        draw_icon(p, Rect::from_center_size(pos2(right - 470.0 + i as f32 * 26.0, mid), Vec2::splat(16.0)), *icon);
    }
    for (i, label) in ["Auto", "Set Key"].iter().enumerate() {
        let b = Rect::from_min_size(pos2(right - 320.0, r.top() + i as f32 * (half + 2.0)), vec2(52.0, half));
        p.rect_filled(b, 2.0, BUTTON);
        p.text(b.center(), Align2::CENTER_CENTER, *label, FontId::proportional(12.0), INK);
    }
    let sel = Rect::from_min_size(pos2(right - 260.0, r.top()), vec2(120.0, half));
    p.rect_filled(sel, 0.0, FIELD2);
    p.text(sel.left_center() + vec2(6.0, 0.0), Align2::LEFT_CENTER, "Selected", FontId::proportional(12.0), INK);
}

// A dropdown-looking field. Real dropdowns replace these as each step wires them.
fn field(ui: &mut Ui, text: &str, width: f32, arrow: bool) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 22.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 0.0, FIELD2);
    p.text(rect.left_center() + vec2(6.0, 0.0), Align2::LEFT_CENTER, text, FontId::proportional(12.0), INK);
    if arrow {
        let c = rect.right_center() - vec2(10.0, 0.0);
        p.add(egui::Shape::convex_polygon(vec![c + vec2(-4.0, -2.0), c + vec2(4.0, -2.0), c + vec2(0.0, 3.0)], Color32::from_gray(0xdd), Stroke::NONE));
    }
}

fn separator(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(9.0, 26.0), Sense::hover());
    ui.painter().line_segment([rect.center_top(), rect.center_bottom()], Stroke::new(1.0, SEP));
}

fn rollout(ui: &mut Ui, title: &str, body: impl FnOnce(&mut Ui)) {
    egui::Frame::new().fill(ROLLOUT).corner_radius(2).inner_margin(egui::Margin { left: 14, right: 14, top: 6, bottom: 14 }).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(title).strong());
        ui.add_space(4.0);
        body(ui);
    });
    ui.add_space(8.0);
}

fn icon_button(ui: &mut Ui, icon: Icon, on: bool, tip: &str) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(36.0, 34.0), Sense::click());
    let p = ui.painter();
    if on {
        p.rect_filled(rect, 0.0, ON);
        p.rect_stroke(rect, 0.0, Stroke::new(1.0, ON_EDGE), egui::StrokeKind::Inside);
    } else if resp.hovered() {
        p.rect_filled(rect, 0.0, Color32::from_gray(0x55));
    }
    draw_icon(p, Rect::from_center_size(rect.center(), Vec2::splat(22.0)), icon);
    resp.on_hover_text(tip)
}

// Our own line icons, drawn with the painter so they stay sharp at any scale.
#[derive(Clone, Copy)]
enum Icon {
    Undo,
    Redo,
    Link,
    Unlink,
    Select,
    RegionRect,
    Move,
    Rotate,
    Scale,
    Snap,
    AngleSnap,
    PercentSnap,
    Mirror,
    Align,
    Layers,
    Explorer,
    RenderSetup,
    Render,
    Create,
    Modify,
    Hierarchy,
    Motion,
    Display,
    Utilities,
    Geometry,
    Shapes,
    Lights,
    Cameras,
    Helpers,
    SpaceWarps,
    Systems,
    GoStart,
    PrevFrame,
    Play,
    NextFrame,
    GoEnd,
}

const CMD_ICONS: [Icon; 6] = [Icon::Create, Icon::Modify, Icon::Hierarchy, Icon::Motion, Icon::Display, Icon::Utilities];
const KIND_ICONS: [Icon; 7] = [Icon::Geometry, Icon::Shapes, Icon::Lights, Icon::Cameras, Icon::Helpers, Icon::SpaceWarps, Icon::Systems];

fn draw_icon(p: &egui::Painter, r: Rect, icon: Icon) {
    let s = Stroke::new(1.8, Color32::from_gray(0xe6));
    let fill = Color32::from_gray(0xe6);
    let at = |x: f32, y: f32| -> Pos2 { pos2(r.left() + x * r.width(), r.top() + y * r.height()) };
    let line = |a: Pos2, b: Pos2| p.line_segment([a, b], s);
    let tri = |pts: [Pos2; 3]| p.add(egui::Shape::convex_polygon(pts.to_vec(), fill, Stroke::NONE));
    let arc = |c: Pos2, rad: f32, from: f32, to: f32| {
        let pts: Vec<Pos2> = (0..=16).map(|i| {
            let a = from + (to - from) * i as f32 / 16.0;
            c + vec2(a.cos(), a.sin()) * rad
        }).collect();
        p.add(egui::Shape::line(pts, s));
    };
    let w = r.width();
    use std::f32::consts::PI;
    match icon {
        Icon::Undo | Icon::Redo => {
            let flip = matches!(icon, Icon::Redo);
            let fx = |x: f32| if flip { 1.0 - x } else { x };
            arc(at(0.5, 0.55), w * 0.3, if flip { PI * 1.1 } else { -PI * 0.1 }, if flip { PI * 2.0 } else { -PI * 1.0 });
            tri([at(fx(0.05), 0.55), at(fx(0.35), 0.55), at(fx(0.2), 0.8)]);
        }
        Icon::Link | Icon::Unlink => {
            p.rect_stroke(Rect::from_min_size(at(0.05, 0.35), vec2(w * 0.4, w * 0.3)), 6.0, s, egui::StrokeKind::Middle);
            p.rect_stroke(Rect::from_min_size(at(0.55, 0.35), vec2(w * 0.4, w * 0.3)), 6.0, s, egui::StrokeKind::Middle);
            if matches!(icon, Icon::Link) {
                line(at(0.3, 0.5), at(0.7, 0.5));
            } else {
                line(at(0.2, 0.9), at(0.8, 0.1));
            }
        }
        Icon::Select => {
            p.add(egui::Shape::convex_polygon(vec![at(0.25, 0.05), at(0.25, 0.8), at(0.45, 0.62), at(0.75, 0.62)], fill, Stroke::NONE));
            line(at(0.45, 0.62), at(0.6, 0.95));
        }
        Icon::RegionRect => {
            for (a, b) in [((0.05, 0.1), (0.95, 0.1)), ((0.95, 0.1), (0.95, 0.9)), ((0.95, 0.9), (0.05, 0.9)), ((0.05, 0.9), (0.05, 0.1))] {
                p.add(egui::Shape::dashed_line(&[at(a.0, a.1), at(b.0, b.1)], s, 3.0, 2.0));
            }
        }
        Icon::Move => {
            line(at(0.5, 0.05), at(0.5, 0.95));
            line(at(0.05, 0.5), at(0.95, 0.5));
            tri([at(0.5, 0.0), at(0.38, 0.18), at(0.62, 0.18)]);
            tri([at(0.5, 1.0), at(0.38, 0.82), at(0.62, 0.82)]);
            tri([at(0.0, 0.5), at(0.18, 0.38), at(0.18, 0.62)]);
            tri([at(1.0, 0.5), at(0.82, 0.38), at(0.82, 0.62)]);
        }
        Icon::Rotate => {
            arc(at(0.5, 0.5), w * 0.4, -PI * 0.3, PI * 1.4);
            tri([at(0.75, 0.05), at(0.95, 0.3), at(0.65, 0.32)]);
        }
        Icon::Scale => {
            p.rect_stroke(Rect::from_min_max(at(0.1, 0.3), at(0.7, 0.9)), 0.0, s, egui::StrokeKind::Middle);
            p.rect_filled(Rect::from_min_max(at(0.1, 0.55), at(0.45, 0.9)), 0.0, fill);
            line(at(0.5, 0.5), at(0.92, 0.08));
            tri([at(0.95, 0.05), at(0.65, 0.1), at(0.9, 0.35)]);
        }
        Icon::Snap | Icon::AngleSnap | Icon::PercentSnap => {
            // A magnet, plus a small mark saying which snap it is.
            arc(at(0.4, 0.45), w * 0.28, PI * 0.0, PI * 1.0);
            line(at(0.12, 0.45), at(0.12, 0.1));
            line(at(0.68, 0.45), at(0.68, 0.1));
            let mark = match icon {
                Icon::Snap => "3",
                Icon::AngleSnap => "∠",
                _ => "%",
            };
            p.text(at(0.88, 0.85), Align2::CENTER_CENTER, mark, FontId::proportional(w * 0.5), fill);
        }
        Icon::Mirror => {
            p.add(egui::Shape::dashed_line(&[at(0.5, 0.0), at(0.5, 1.0)], s, 3.0, 2.0));
            tri([at(0.4, 0.15), at(0.4, 0.85), at(0.05, 0.85)]);
            p.add(egui::Shape::closed_line(vec![at(0.6, 0.15), at(0.6, 0.85), at(0.95, 0.85)], s));
        }
        Icon::Align => {
            line(at(0.1, 0.05), at(0.1, 0.95));
            p.rect_filled(Rect::from_min_max(at(0.2, 0.15), at(0.9, 0.4)), 0.0, fill);
            p.rect_filled(Rect::from_min_max(at(0.2, 0.6), at(0.6, 0.85)), 0.0, fill);
        }
        Icon::Layers => {
            for y in [0.25, 0.5, 0.75] {
                p.add(egui::Shape::closed_line(vec![at(0.5, y - 0.18), at(0.95, y), at(0.5, y + 0.18), at(0.05, y)], s));
            }
        }
        Icon::Explorer | Icon::Hierarchy => {
            p.rect_filled(Rect::from_min_max(at(0.05, 0.05), at(0.35, 0.3)), 0.0, fill);
            line(at(0.2, 0.3), at(0.2, 0.8));
            line(at(0.2, 0.5), at(0.5, 0.5));
            line(at(0.2, 0.8), at(0.5, 0.8));
            p.rect_filled(Rect::from_min_max(at(0.5, 0.38), at(0.95, 0.62)), 0.0, fill);
            p.rect_filled(Rect::from_min_max(at(0.5, 0.68), at(0.95, 0.92)), 0.0, fill);
        }
        Icon::RenderSetup => {
            p.rect_stroke(Rect::from_min_max(at(0.05, 0.1), at(0.95, 0.9)), 1.0, s, egui::StrokeKind::Middle);
            for (y, x) in [(0.35, 0.3), (0.65, 0.65)] {
                line(at(0.2, y), at(0.8, y));
                p.circle_filled(at(x, y), w * 0.08, fill);
            }
        }
        Icon::Render => {
            p.add(egui::Shape::closed_line(vec![at(0.1, 0.35), at(0.5, 0.15), at(0.9, 0.35), at(0.9, 0.85), at(0.1, 0.85)], s));
            p.circle_filled(at(0.5, 0.58), w * 0.15, fill);
        }
        Icon::Create => {
            line(at(0.5, 0.05), at(0.5, 0.95));
            line(at(0.05, 0.5), at(0.95, 0.5));
        }
        Icon::Modify => {
            p.rect_stroke(Rect::from_min_max(at(0.1, 0.1), at(0.9, 0.9)), 0.0, s, egui::StrokeKind::Middle);
            line(at(0.1, 0.9), at(0.9, 0.1));
        }
        Icon::Motion => {
            p.circle_filled(at(0.7, 0.5), w * 0.22, fill);
            for (i, y) in [0.3, 0.5, 0.7].iter().enumerate() {
                line(at(0.05 + i as f32 * 0.05, *y), at(0.4, *y));
            }
        }
        Icon::Display => {
            p.rect_stroke(Rect::from_min_max(at(0.05, 0.1), at(0.95, 0.7)), 1.0, s, egui::StrokeKind::Middle);
            line(at(0.5, 0.7), at(0.5, 0.9));
            line(at(0.25, 0.92), at(0.75, 0.92));
        }
        Icon::Utilities => {
            line(at(0.15, 0.85), at(0.65, 0.35));
            arc(at(0.72, 0.28), w * 0.2, PI * 0.9, PI * 2.6);
        }
        Icon::Geometry => {
            p.circle_filled(at(0.5, 0.5), w * 0.45, fill);
        }
        Icon::Shapes => {
            p.add(egui::Shape::closed_line(vec![at(0.1, 0.9), at(0.1, 0.4), at(0.6, 0.1), at(0.9, 0.5), at(0.6, 0.9)], s));
        }
        Icon::Lights => {
            p.circle_filled(at(0.5, 0.38), w * 0.3, fill);
            p.rect_filled(Rect::from_min_max(at(0.38, 0.7), at(0.62, 0.95)), 0.0, fill);
        }
        Icon::Cameras => {
            p.rect_filled(Rect::from_min_max(at(0.05, 0.3), at(0.65, 0.8)), 1.0, fill);
            tri([at(0.65, 0.55), at(0.95, 0.3), at(0.95, 0.8)]);
        }
        Icon::Helpers => {
            line(at(0.05, 0.95), at(0.95, 0.95));
            line(at(0.05, 0.95), at(0.05, 0.05));
            line(at(0.05, 0.95), at(0.7, 0.3));
        }
        Icon::SpaceWarps => {
            for y in [0.3, 0.55, 0.8] {
                let pts: Vec<Pos2> = (0..=12).map(|i| at(0.05 + i as f32 * 0.075, y + 0.1 * (i as f32 * 0.9).sin())).collect();
                p.add(egui::Shape::line(pts, s));
            }
        }
        Icon::Systems => {
            p.circle_stroke(at(0.3, 0.3), w * 0.18, s);
            p.circle_stroke(at(0.7, 0.7), w * 0.18, s);
            line(at(0.42, 0.42), at(0.58, 0.58));
        }
        Icon::GoStart | Icon::GoEnd => {
            let flip = matches!(icon, Icon::GoEnd);
            let fx = |x: f32| if flip { 1.0 - x } else { x };
            p.rect_filled(Rect::from_min_max(at(fx(0.05).min(fx(0.2)), 0.1), at(fx(0.05).max(fx(0.2)), 0.9)), 0.0, fill);
            tri([at(fx(0.2), 0.5), at(fx(0.95), 0.1), at(fx(0.95), 0.9)]);
        }
        Icon::PrevFrame | Icon::NextFrame => {
            let flip = matches!(icon, Icon::NextFrame);
            let fx = |x: f32| if flip { 1.0 - x } else { x };
            tri([at(fx(0.1), 0.5), at(fx(0.6), 0.1), at(fx(0.6), 0.9)]);
            p.rect_filled(Rect::from_min_max(at(fx(0.7).min(fx(0.9)), 0.1), at(fx(0.7).max(fx(0.9)), 0.9)), 0.0, fill);
        }
        Icon::Play => {
            tri([at(0.15, 0.05), at(0.15, 0.95), at(0.95, 0.5)]);
        }
    }
}
