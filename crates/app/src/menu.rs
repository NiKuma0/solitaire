//! The settings screen (SPEC §6.4), laid out with taffy.
//!
//! The title bar stays put; the rows below it scroll when the screen is short.
//! Text widths are estimated from character counts so the layout needs no
//! window and can be tested.

use macroquad::math::{Rect, Vec2, vec2};
use solitaire_core::DrawMode;
use taffy::prelude::{
    AlignItems, AvailableSpace, Dimension, FlexDirection, FlexWrap, LengthPercentage,
    LengthPercentageAuto, NodeId, Size, Style, TaffyTree,
};

use crate::layout::Insets;
use crate::settings::{AnimSpeed, Settings};
use crate::theme::ThemeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    Draw,
    Theme,
    FourColor,
    Animation,
    TapToMove,
    LeftHanded,
    LargeCards,
    ResetStats,
}

enum Control {
    Choice(&'static [&'static str]),
    Toggle,
    Action(&'static str),
}

impl Row {
    pub const ALL: [Self; 8] = [
        Self::Draw,
        Self::Theme,
        Self::FourColor,
        Self::Animation,
        Self::TapToMove,
        Self::LeftHanded,
        Self::LargeCards,
        Self::ResetStats,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Draw => "Draw",
            Self::Theme => "Theme",
            Self::FourColor => "Four-colour suits",
            Self::Animation => "Animations",
            Self::TapToMove => "Tap to move",
            Self::LeftHanded => "Left-handed layout",
            Self::LargeCards => "Large cards",
            Self::ResetStats => "Statistics",
        }
    }

    const fn control(self) -> Control {
        match self {
            Self::Draw => Control::Choice(&["1 card", "3 cards"]),
            Self::Theme => Control::Choice(&["Light", "Dark", "Black", "Warm", "High contrast"]),
            Self::Animation => Control::Choice(&["Off", "Fast", "Normal"]),
            Self::FourColor | Self::TapToMove | Self::LeftHanded | Self::LargeCards => {
                Control::Toggle
            }
            Self::ResetStats => Control::Action("Reset"),
        }
    }

    /// Option labels for a choice row (empty for toggles and actions).
    #[must_use]
    pub fn options(self) -> &'static [&'static str] {
        match self.control() {
            Control::Choice(options) => options,
            Control::Toggle | Control::Action(_) => &[],
        }
    }

    /// Button text for an action row.
    #[must_use]
    pub const fn action(self) -> Option<&'static str> {
        match self.control() {
            Control::Action(text) => Some(text),
            Control::Choice(_) | Control::Toggle => None,
        }
    }

    #[must_use]
    pub const fn is_toggle(self) -> bool {
        matches!(self.control(), Control::Toggle)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuHit {
    Done,
    Choose(Row, usize),
    Toggle(Row),
    Action(Row),
}

/// A row in screen coordinates at zero scroll.
#[derive(Debug, Clone, PartialEq)]
pub struct RowLayout {
    pub row: Row,
    pub bounds: Rect,
    pub label: Rect,
    /// One rect per option, or the switch / button.
    pub targets: Vec<Rect>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Menu {
    pub panel: Rect,
    pub title: Rect,
    pub done: Rect,
    /// The scrolling part of the panel.
    pub viewport: Rect,
    pub rows: Vec<RowLayout>,
    /// Text block with version, licences and source link.
    pub about: Rect,
    content_height: f32,
}

pub const ABOUT_LINES: [&str; 4] = [
    concat!("Solitaire ", env!("CARGO_PKG_VERSION")),
    "Free software: MIT OR Apache-2.0",
    "github.com/NiKuma0/solitaire",
    "Font: Inter, SIL Open Font License 1.1",
];

pub const LABEL_SIZE: f32 = 16.0;
pub const ABOUT_LINE: f32 = 22.0;
const SCREEN_MARGIN: f32 = 12.0;
const MAX_PANEL_WIDTH: f32 = 520.0;
const TITLE_HEIGHT: f32 = 56.0;
const PADDING: f32 = 16.0;
const TARGET: f32 = 44.0;
const SWITCH_WIDTH: f32 = 56.0;
const ACTION_WIDTH: f32 = 88.0;

/// Generous width of `text` at `size`: Inter averages about 0.6 em per
/// character, so this rather overestimates than lets text spill out.
fn text_width(text: &str, size: f32) -> f32 {
    text.chars().count() as f32 * size * 0.62
}

fn option_width(text: &str) -> f32 {
    (text_width(text, LABEL_SIZE) + 28.0).max(48.0)
}

impl Menu {
    #[must_use]
    pub fn new(screen: Vec2, insets: Insets) -> Self {
        let area = Rect::new(
            insets.left + SCREEN_MARGIN,
            insets.top + SCREEN_MARGIN,
            (screen.x - insets.left - insets.right - 2.0 * SCREEN_MARGIN).max(TARGET),
            (screen.y - insets.top - insets.bottom - 2.0 * SCREEN_MARGIN).max(TITLE_HEIGHT),
        );
        let width = area.w.min(MAX_PANEL_WIDTH);
        let (rows, about, content_height) = content(width).unwrap_or_default();
        let height = (TITLE_HEIGHT + content_height).min(area.h);
        let panel = Rect::new(
            area.center().x - width / 2.0,
            area.center().y - height / 2.0,
            width,
            height,
        );
        let title = Rect::new(panel.x, panel.y, panel.w, TITLE_HEIGHT);
        let done = Rect::new(
            panel.right() - PADDING - 72.0,
            panel.y + (TITLE_HEIGHT - TARGET) / 2.0,
            72.0,
            TARGET,
        );
        let viewport = Rect::new(
            panel.x,
            title.bottom(),
            panel.w,
            (panel.h - TITLE_HEIGHT).max(0.0),
        );
        let origin = vec2(panel.x, title.bottom());
        let shift = |r: Rect| r.offset(origin);
        Self {
            panel,
            title,
            done,
            viewport,
            rows: rows
                .into_iter()
                .map(|r| RowLayout {
                    row: r.row,
                    bounds: shift(r.bounds),
                    label: shift(r.label),
                    targets: r.targets.into_iter().map(shift).collect(),
                })
                .collect(),
            about: shift(about),
            content_height,
        }
    }

    #[must_use]
    pub fn max_scroll(&self) -> f32 {
        (self.content_height - self.viewport.h).max(0.0)
    }

    /// What is under `p` when the rows are scrolled up by `scroll`.
    #[must_use]
    pub fn hit(&self, p: Vec2, scroll: f32) -> Option<MenuHit> {
        if self.done.contains(p) {
            return Some(MenuHit::Done);
        }
        if !self.viewport.contains(p) {
            return None;
        }
        let q = p + vec2(0.0, scroll);
        self.rows.iter().find_map(|r| {
            if let Some(i) = r.targets.iter().position(|t| t.contains(q))
                && !r.row.is_toggle()
                && r.row.action().is_none()
            {
                return Some(MenuHit::Choose(r.row, i));
            }
            if !r.bounds.contains(q) {
                return None;
            }
            if r.row.is_toggle() {
                Some(MenuHit::Toggle(r.row))
            } else if r.row.action().is_some() && r.targets.iter().any(|t| t.contains(q)) {
                Some(MenuHit::Action(r.row))
            } else {
                None
            }
        })
    }
}

type Tree = TaffyTree<()>;

/// Taffy nodes for one row: the row, its label, its control and the control's targets.
struct RowNodes {
    row: Row,
    node: NodeId,
    label: NodeId,
    control: NodeId,
    targets: Vec<NodeId>,
}

fn px(v: f32) -> LengthPercentage {
    LengthPercentage::length(v)
}

fn fixed(w: f32, h: f32) -> Style {
    Style {
        size: Size {
            width: Dimension::length(w),
            height: Dimension::length(h),
        },
        flex_shrink: 0.0,
        ..Style::default()
    }
}

/// A row that wraps its children onto more lines when they do not fit.
fn wrap_row(gap: f32) -> Style {
    Style {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::CENTER,
        gap: Size {
            width: px(gap),
            height: px(6.0),
        },
        ..Style::default()
    }
}

fn row_nodes(tree: &mut Tree, row: Row) -> Result<RowNodes, taffy::TaffyError> {
    let label = tree.new_leaf(Style {
        flex_grow: 1.0,
        flex_basis: Dimension::length(text_width(row.label(), LABEL_SIZE)),
        min_size: Size {
            width: LengthPercentageAuto::auto(),
            height: LengthPercentageAuto::length(TARGET),
        },
        ..Style::default()
    })?;
    let targets: Vec<NodeId> = match row.control() {
        Control::Choice(options) => options
            .iter()
            .map(|o| tree.new_leaf(fixed(option_width(o), TARGET)))
            .collect::<Result<_, _>>()?,
        Control::Toggle => vec![tree.new_leaf(fixed(SWITCH_WIDTH, TARGET))?],
        Control::Action(_) => vec![tree.new_leaf(fixed(ACTION_WIDTH, TARGET))?],
    };
    let control = tree.new_with_children(wrap_row(6.0), &targets)?;
    let style = Style {
        padding: taffy::Rect {
            left: px(0.0),
            right: px(0.0),
            top: px(4.0),
            bottom: px(4.0),
        },
        ..wrap_row(12.0)
    };
    let node = tree.new_with_children(style, &[label, control])?;
    Ok(RowNodes {
        row,
        node,
        label,
        control,
        targets,
    })
}

/// Lays the rows and the about block out in panel coordinates.
fn content(width: f32) -> Result<(Vec<RowLayout>, Rect, f32), taffy::TaffyError> {
    let mut tree = Tree::new();
    let rows: Vec<RowNodes> = Row::ALL
        .iter()
        .map(|&r| row_nodes(&mut tree, r))
        .collect::<Result<_, _>>()?;
    let about = tree.new_leaf(fixed(
        width - 2.0 * PADDING,
        ABOUT_LINE * ABOUT_LINES.len() as f32 + 16.0,
    ))?;
    let mut children: Vec<NodeId> = rows.iter().map(|r| r.node).collect();
    children.push(about);
    let root = tree.new_with_children(
        Style {
            flex_direction: FlexDirection::Column,
            size: Size {
                width: Dimension::length(width),
                height: Dimension::auto(),
            },
            padding: taffy::Rect {
                left: px(PADDING),
                right: px(PADDING),
                top: px(4.0),
                bottom: px(PADDING),
            },
            ..Style::default()
        },
        &children,
    )?;
    tree.compute_layout(
        root,
        Size {
            width: AvailableSpace::Definite(width),
            height: AvailableSpace::MaxContent,
        },
    )?;

    let rect = |node: NodeId, origin: Vec2| -> Result<Rect, taffy::TaffyError> {
        let l = tree.layout(node)?;
        Ok(Rect::new(
            origin.x + l.location.x,
            origin.y + l.location.y,
            l.size.width,
            l.size.height,
        ))
    };
    let mut out = Vec::new();
    for r in rows {
        let bounds = rect(r.node, Vec2::ZERO)?;
        let control = rect(r.control, bounds.point())?;
        out.push(RowLayout {
            row: r.row,
            bounds,
            label: rect(r.label, bounds.point())?,
            targets: r
                .targets
                .into_iter()
                .map(|t| rect(t, control.point()))
                .collect::<Result<_, _>>()?,
        });
    }
    let about = rect(about, Vec2::ZERO)?;
    let height = tree.layout(root)?.size.height;
    Ok((out, about, height))
}

/// Index of the current option, or 0/1 for toggles; `None` for actions.
#[must_use]
pub fn selected(s: &Settings, row: Row) -> Option<usize> {
    Some(match row {
        Row::Draw => usize::from(s.draw == DrawMode::Three),
        Row::Theme => ThemeId::ALL.iter().position(|&t| t == s.theme)?,
        Row::Animation => [AnimSpeed::Off, AnimSpeed::Fast, AnimSpeed::Normal]
            .iter()
            .position(|&a| a == s.animation)?,
        Row::FourColor => usize::from(s.four_color),
        Row::TapToMove => usize::from(s.tap_to_move),
        Row::LeftHanded => usize::from(s.left_handed),
        Row::LargeCards => usize::from(s.large_cards),
        Row::ResetStats => return None,
    })
}

/// Applies a tap to the settings; true if anything changed.
pub fn apply(s: &mut Settings, hit: MenuHit) -> bool {
    let before = *s;
    match hit {
        MenuHit::Choose(Row::Draw, i) => {
            s.draw = if i == 1 {
                DrawMode::Three
            } else {
                DrawMode::One
            }
        }
        MenuHit::Choose(Row::Theme, i) => {
            if let Some(&t) = ThemeId::ALL.get(i) {
                s.theme = t;
            }
        }
        MenuHit::Choose(Row::Animation, i) => {
            if let Some(&a) = [AnimSpeed::Off, AnimSpeed::Fast, AnimSpeed::Normal].get(i) {
                s.animation = a;
            }
        }
        MenuHit::Toggle(Row::FourColor) => s.four_color = !s.four_color,
        MenuHit::Toggle(Row::TapToMove) => s.tap_to_move = !s.tap_to_move,
        MenuHit::Toggle(Row::LeftHanded) => s.left_handed = !s.left_handed,
        MenuHit::Toggle(Row::LargeCards) => s.large_cards = !s.large_cards,
        // Statistics arrive in M4; until then reset does nothing.
        MenuHit::Action(_) | MenuHit::Done | MenuHit::Choose(..) | MenuHit::Toggle(_) => {}
    }
    *s != before
}

#[cfg(test)]
#[path = "tests/menu.rs"]
mod tests;
