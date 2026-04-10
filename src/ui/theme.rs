use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeName {
    #[default]
    Catppuccin,
    Gruvbox,
    TokyoNight,
    Nord,
    Dracula,
    Terminal,
    NoColor,
}

impl ThemeName {
    pub const ALL: [Self; 7] = [
        Self::Catppuccin,
        Self::Gruvbox,
        Self::TokyoNight,
        Self::Nord,
        Self::Dracula,
        Self::Terminal,
        Self::NoColor,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Catppuccin => "catppuccin",
            Self::Gruvbox => "gruvbox",
            Self::TokyoNight => "tokyo-night",
            Self::Nord => "nord",
            Self::Dracula => "dracula",
            Self::Terminal => "terminal",
            Self::NoColor => "no-color",
        }
    }

    pub fn next(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|theme| *theme == self)
            .expect("theme should exist in cycle");
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    pub fn resolve(self) -> Theme {
        match self {
            Self::Catppuccin => palette_theme(Palette {
                background: Some(rgb(30, 30, 46)),
                base: rgb(205, 214, 244),
                muted: rgb(166, 173, 200),
                emphasis: rgb(203, 166, 247),
                border: rgb(88, 91, 112),
                focus_border: rgb(137, 180, 250),
                overlay_border: rgb(116, 199, 236),
                warning: rgb(249, 226, 175),
                danger: rgb(243, 139, 168),
                selected_fg: rgb(30, 30, 46),
                selected_bg: rgb(180, 190, 254),
                working: rgb(137, 220, 235),
                attention: rgb(249, 226, 175),
                idle: rgb(166, 227, 161),
                error: rgb(243, 139, 168),
                unknown: rgb(186, 194, 222),
                non_agent: rgb(127, 132, 156),
            }),
            Self::Gruvbox => palette_theme(Palette {
                background: Some(rgb(40, 40, 40)),
                base: rgb(235, 219, 178),
                muted: rgb(168, 153, 132),
                emphasis: rgb(211, 134, 155),
                border: rgb(124, 111, 100),
                focus_border: rgb(131, 165, 152),
                overlay_border: rgb(250, 189, 47),
                warning: rgb(250, 189, 47),
                danger: rgb(251, 73, 52),
                selected_fg: rgb(40, 40, 40),
                selected_bg: rgb(250, 189, 47),
                working: rgb(142, 192, 124),
                attention: rgb(250, 189, 47),
                idle: rgb(184, 187, 38),
                error: rgb(251, 73, 52),
                unknown: rgb(235, 219, 178),
                non_agent: rgb(146, 131, 116),
            }),
            Self::TokyoNight => palette_theme(Palette {
                background: Some(rgb(26, 27, 38)),
                base: rgb(192, 202, 245),
                muted: rgb(134, 142, 180),
                emphasis: rgb(187, 154, 247),
                border: rgb(68, 74, 106),
                focus_border: rgb(122, 162, 247),
                overlay_border: rgb(125, 207, 255),
                warning: rgb(224, 175, 104),
                danger: rgb(247, 118, 142),
                selected_fg: rgb(26, 27, 38),
                selected_bg: rgb(122, 162, 247),
                working: rgb(125, 207, 255),
                attention: rgb(224, 175, 104),
                idle: rgb(158, 206, 106),
                error: rgb(247, 118, 142),
                unknown: rgb(187, 154, 247),
                non_agent: rgb(86, 95, 137),
            }),
            Self::Nord => palette_theme(Palette {
                background: Some(rgb(46, 52, 64)),
                base: rgb(236, 239, 244),
                muted: rgb(143, 188, 187),
                emphasis: rgb(136, 192, 208),
                border: rgb(76, 86, 106),
                focus_border: rgb(129, 161, 193),
                overlay_border: rgb(94, 129, 172),
                warning: rgb(235, 203, 139),
                danger: rgb(191, 97, 106),
                selected_fg: rgb(46, 52, 64),
                selected_bg: rgb(129, 161, 193),
                working: rgb(94, 129, 172),
                attention: rgb(235, 203, 139),
                idle: rgb(163, 190, 140),
                error: rgb(191, 97, 106),
                unknown: rgb(136, 192, 208),
                non_agent: rgb(129, 161, 193),
            }),
            Self::Dracula => palette_theme(Palette {
                background: Some(rgb(40, 42, 54)),
                base: rgb(248, 248, 242),
                muted: rgb(98, 114, 164),
                emphasis: rgb(255, 121, 198),
                border: rgb(98, 114, 164),
                focus_border: rgb(139, 233, 253),
                overlay_border: rgb(189, 147, 249),
                warning: rgb(241, 250, 140),
                danger: rgb(255, 85, 85),
                selected_fg: rgb(40, 42, 54),
                selected_bg: rgb(189, 147, 249),
                working: rgb(139, 233, 253),
                attention: rgb(241, 250, 140),
                idle: rgb(80, 250, 123),
                error: rgb(255, 85, 85),
                unknown: rgb(189, 147, 249),
                non_agent: rgb(98, 114, 164),
            }),
            Self::Terminal => terminal_theme(),
            Self::NoColor => no_color_theme(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub base: Style,
    pub muted: Style,
    pub emphasis: Style,
    pub border: Style,
    pub focus_border: Style,
    pub overlay_border: Style,
    pub search_border: Style,
    pub warning_border: Style,
    pub danger_border: Style,
    pub selected: Style,
    pub working: Style,
    pub attention: Style,
    pub idle: Style,
    pub error: Style,
    pub unknown: Style,
    pub non_agent: Style,
    pub glyphs: Glyphs,
}

#[derive(Debug, Clone, Copy)]
pub struct Glyphs {
    pub selected: &'static str,
    pub session_open: &'static str,
    pub session_closed: &'static str,
    pub working: &'static str,
    pub attention: &'static str,
    pub idle: &'static str,
    pub error: &'static str,
    pub unknown: &'static str,
    pub non_agent: &'static str,
    pub separator: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct Palette {
    background: Option<Color>,
    base: Color,
    muted: Color,
    emphasis: Color,
    border: Color,
    focus_border: Color,
    overlay_border: Color,
    warning: Color,
    danger: Color,
    selected_fg: Color,
    selected_bg: Color,
    working: Color,
    attention: Color,
    idle: Color,
    error: Color,
    unknown: Color,
    non_agent: Color,
}

fn rgb(red: u8, green: u8, blue: u8) -> Color {
    Color::Rgb(red, green, blue)
}

fn style(color: Color, background: Option<Color>) -> Style {
    let style = Style::default().fg(color);
    if let Some(background) = background {
        style.bg(background)
    } else {
        style
    }
}

fn palette_theme(palette: Palette) -> Theme {
    Theme {
        base: style(palette.base, palette.background),
        muted: style(palette.muted, palette.background),
        emphasis: style(palette.emphasis, palette.background).add_modifier(Modifier::BOLD),
        border: Style::default().fg(palette.border),
        focus_border: Style::default()
            .fg(palette.focus_border)
            .add_modifier(Modifier::BOLD),
        overlay_border: Style::default()
            .fg(palette.overlay_border)
            .add_modifier(Modifier::BOLD),
        search_border: Style::default()
            .fg(palette.warning)
            .add_modifier(Modifier::BOLD),
        warning_border: Style::default()
            .fg(palette.warning)
            .add_modifier(Modifier::BOLD),
        danger_border: Style::default()
            .fg(palette.danger)
            .add_modifier(Modifier::BOLD),
        selected: Style::default()
            .fg(palette.selected_fg)
            .bg(palette.selected_bg)
            .add_modifier(Modifier::BOLD),
        working: style(palette.working, palette.background).add_modifier(Modifier::BOLD),
        attention: style(palette.attention, palette.background).add_modifier(Modifier::BOLD),
        idle: style(palette.idle, palette.background).add_modifier(Modifier::BOLD),
        error: style(palette.error, palette.background).add_modifier(Modifier::BOLD),
        unknown: style(palette.unknown, palette.background).add_modifier(Modifier::BOLD),
        non_agent: style(palette.non_agent, palette.background),
        glyphs: unicode_glyphs(),
    }
}

fn terminal_theme() -> Theme {
    Theme {
        base: Style::default(),
        muted: Style::default().fg(Color::DarkGray),
        emphasis: Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        border: Style::default().fg(Color::Blue),
        focus_border: Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        overlay_border: Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        search_border: Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        warning_border: Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        danger_border: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        selected: Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED),
        working: Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD),
        attention: Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        idle: Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        error: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        unknown: Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
        non_agent: Style::default().fg(Color::DarkGray),
        glyphs: unicode_glyphs(),
    }
}

fn no_color_theme() -> Theme {
    Theme {
        base: Style::default(),
        muted: Style::default(),
        emphasis: Style::default().add_modifier(Modifier::BOLD),
        border: Style::default(),
        focus_border: Style::default().add_modifier(Modifier::BOLD),
        overlay_border: Style::default().add_modifier(Modifier::BOLD),
        search_border: Style::default().add_modifier(Modifier::BOLD),
        warning_border: Style::default().add_modifier(Modifier::BOLD),
        danger_border: Style::default().add_modifier(Modifier::BOLD),
        selected: Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED),
        working: Style::default().add_modifier(Modifier::BOLD),
        attention: Style::default().add_modifier(Modifier::BOLD),
        idle: Style::default().add_modifier(Modifier::BOLD),
        error: Style::default().add_modifier(Modifier::BOLD),
        unknown: Style::default().add_modifier(Modifier::BOLD),
        non_agent: Style::default(),
        glyphs: ascii_glyphs(),
    }
}

fn unicode_glyphs() -> Glyphs {
    Glyphs {
        selected: "›",
        session_open: "▾",
        session_closed: "▸",
        working: "~",
        attention: "!",
        idle: "•",
        error: "×",
        unknown: "?",
        non_agent: "·",
        separator: "•",
    }
}

fn ascii_glyphs() -> Glyphs {
    Glyphs {
        selected: ">",
        session_open: "v",
        session_closed: ">",
        working: "~",
        attention: "!",
        idle: ".",
        error: "x",
        unknown: "?",
        non_agent: "-",
        separator: "|",
    }
}

#[cfg(test)]
mod tests {
    use super::ThemeName;

    #[test]
    fn theme_cycle_visits_every_theme_before_wrapping() {
        let mut order = Vec::new();
        let mut theme = ThemeName::Catppuccin;

        for _ in 0..ThemeName::ALL.len() {
            order.push(theme);
            theme = theme.next();
        }

        assert_eq!(order, ThemeName::ALL);
        assert_eq!(theme, ThemeName::Catppuccin);
    }

    #[test]
    fn no_color_theme_uses_ascii_glyphs() {
        let glyphs = ThemeName::NoColor.resolve().glyphs;

        assert_eq!(glyphs.selected, ">");
        assert_eq!(glyphs.separator, "|");
        assert_eq!(glyphs.error, "x");
    }
}
