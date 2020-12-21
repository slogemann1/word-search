use iced::widget::{
    button,
    text_input,
    container,
    pick_list,
};
use iced::Color;

const BACKGROUND: Color = Color::from_rgb(
    0x2C as f32 / 255.0,
    0x2C as f32 / 255.0,
    0x2C as f32 / 255.0,
);

const ACCENT: Color = Color::from_rgb(
    0xD0 as f32 / 255.0,
    0xA4 as f32 / 255.0,
    0xFF as f32 / 255.0
);

const CONTRAST: Color = Color::from_rgb(
    0xB0 as f32 / 255.0,
    0xD9 as f32 / 255.0,
    0xDF as f32 / 255.0
);

const CONTRAST_LIGHTER: Color = Color::from_rgb(
    0xC8 as f32 / 255.0,
    0xC7 as f32 / 255.0,
    0xF7 as f32 / 255.0,
);

const WIDGET_BACKGROUND: Color = Color::from_rgb(
    0x40 as f32 / 255.0,
    0x40 as f32 / 255.0,
    0x40 as f32 / 255.0,
);

const WIDGET_HIGHLIGHT: Color = Color::from_rgb(
    0x35 as f32 / 255.0,
    0x35 as f32 / 255.0,
    0x35 as f32 / 255.0,
);

const WIDGET_LIGHTER: Color = Color::from_rgb(
    0x99 as f32 / 255.0,
    0x99 as f32 / 255.0,
    0x99 as f32 / 255.0,
);

const WIDGET_TEXT: Color = Color::from_rgb(
    0xC5 as f32 / 255.0,
    0xB5 as f32 / 255.0,
    0xFB as f32 / 255.0
);

pub struct MyStyle;

impl From<MyStyle> for Box<dyn button::StyleSheet> {
    fn from(_: MyStyle) -> Self { 
        Button.into()
    }
}

impl From<MyStyle> for Box<dyn text_input::StyleSheet> {
    fn from(_: MyStyle) -> Self {
        TextInput.into()
    }
}

impl From<MyStyle> for Box<dyn container::StyleSheet> {
    fn from(_: MyStyle) -> Self { 
        Container.into()
    }
}

impl From<MyStyle> for Box<dyn pick_list::StyleSheet> {
    fn from(_: MyStyle) -> Self { 
        PickList.into()
    }
}

struct Button;
struct TextInput;
struct Container;
struct PickList;

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        button::Style {
            background: WIDGET_BACKGROUND.into(),
            border_radius: 5.0,
            border_color: ACCENT,
            border_width: 1.0,
            text_color: WIDGET_TEXT,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: WIDGET_HIGHLIGHT.into(),
            border_color: ACCENT,
            border_radius: 5.0,
            border_width: 1.0,
            text_color: WIDGET_TEXT,
            ..button::Style::default()
        }
    }
}

impl container::StyleSheet for Container {
    fn style(&self) -> container::Style {
        container::Style {
            background: BACKGROUND.into(),
            text_color: CONTRAST.into(),
            ..container::Style::default()
        }
    }
}

impl text_input::StyleSheet for TextInput {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: WIDGET_LIGHTER.into(),
            border_radius: 2.0,
            border_width: 0.0,
            border_color: ACCENT,
        }
    }

    fn placeholder_color(&self) -> Color {
        CONTRAST_LIGHTER
    }

    fn value_color(&self) -> Color {
        Color::from_rgb(
            0x58 as f32 / 255.0,
            0x57 as f32 / 255.0,
            0x87 as f32 / 255.0,
        )
    }

    fn selection_color(&self) -> Color {
        Color::WHITE
    }

    fn focused(&self) -> text_input::Style {
        self.active()
    }
}

impl pick_list::StyleSheet for PickList {
    fn active(&self) -> pick_list::Style {
        pick_list::Style {
            text_color: WIDGET_TEXT,
            background: WIDGET_BACKGROUND.into(),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: ACCENT,
            ..pick_list::Style::default()
        }
    }

    fn menu(&self) -> pick_list::Menu {
        pick_list::Menu {
            text_color: WIDGET_TEXT,
            background: WIDGET_BACKGROUND.into(),
            border_width: 0.5,
            border_color: ACCENT,
            selected_text_color: CONTRAST,
            selected_background: WIDGET_HIGHLIGHT.into()
        }
    }

    fn hovered(&self) -> pick_list::Style {
        self.active()
    }
}