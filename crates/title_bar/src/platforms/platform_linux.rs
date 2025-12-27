use std::sync::Arc;
use gpui::{img, prelude::*, Action, Global, ImageSource, MouseButton, Resource, WindowAppearance};
use ui::prelude::*;

#[derive(IntoElement)]
pub struct LinuxWindowControls {
    close_window_action: Box<dyn Action>,
}

impl LinuxWindowControls {
    pub fn new(close_window_action: Box<dyn Action>) -> Self {
        Self {
            close_window_action,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct ControlsState {
    minimize_control_state: WindowControlState,
    maximize_or_restore_control_state: WindowControlState,
    close_control_state: WindowControlState,
}

impl Default for ControlsState {
    fn default() -> Self {
        Self {
            minimize_control_state: WindowControlState::Normal,
            maximize_or_restore_control_state: WindowControlState::Normal,
            close_control_state: WindowControlState::Normal,
        }
    }
}

impl Global for ControlsState {

}

impl RenderOnce for LinuxWindowControls {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let controls_state = cx.default_global::<ControlsState>();

        let ControlsState {
            minimize_control_state,
            maximize_or_restore_control_state,
            close_control_state
        } = controls_state.clone();

        h_flex()
            .id("generic-window-controls")
            .px_3()
            .gap_2()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(WindowControl::new(
                "minimize",
                WindowControlType::Minimize,
                minimize_control_state,
                cx,
            ))
            .child(WindowControl::new(
                "maximize-or-restore",
                if window.is_maximized() {
                    WindowControlType::Restore
                } else {
                    WindowControlType::Maximize
                },
                maximize_or_restore_control_state,
                cx,
            ))
            .child(WindowControl::new_close(
                "close",
                WindowControlType::Close,
                close_control_state,
                self.close_window_action,
                cx,
            ))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum WindowControlType {
    Minimize,
    Restore,
    Maximize,
    Close,
}

impl WindowControlType {
    pub fn name(&self) -> Arc<str> {
        match self {
            WindowControlType::Minimize => "minimize".into(),
            WindowControlType::Restore => "restore".into(),
            WindowControlType::Maximize => "maximize".into(),
            WindowControlType::Close => "close".into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum WindowControlState {
    Normal,
    Hover,
    Active,
    Disable,
}

impl WindowControlState {
    pub fn name(&self) -> Arc<str> {
        match self {
            WindowControlState::Normal => "normal".into(),
            WindowControlState::Hover => "hover".into(),
            WindowControlState::Active => "active".into(),
            WindowControlState::Disable => "disable".into(),
        }
    }
}

#[derive(IntoElement)]
pub struct WindowControl {
    id: ElementId,
    control_type: WindowControlType,
    control_state: WindowControlState,
    close_action: Option<Box<dyn Action>>,
}

impl WindowControl {
    pub fn new(id: impl Into<ElementId>, control_type: WindowControlType, control_state: WindowControlState, _cx: &mut App) -> Self {

        Self {
            id: id.into(),
            control_type,
            control_state,
            close_action: None,
        }
    }

    pub fn new_close(
        id: impl Into<ElementId>,
        control_type: WindowControlType,
        control_state: WindowControlState,
        close_action: Box<dyn Action>,
        _cx: &mut App,
    ) -> Self {

        Self {
            id: id.into(),
            control_type,
            control_state,
            close_action: Some(close_action.boxed_clone()),
        }
    }

    fn icon(&self, window_active: bool, appearance: WindowAppearance) -> String {
        let style = match appearance {
            WindowAppearance::Light => "light",
            WindowAppearance::VibrantLight => "light",
            WindowAppearance::Dark => "dark",
            WindowAppearance::VibrantDark => "dark",
        };
        if !window_active || self.control_state == WindowControlState::Disable {
            format!("icons/window_controls/backdrop-{}.svg", style)
        } else {
            let type_name = self.control_type.name();
            let state_name = self.control_state.name();
            format!("icons/window_controls/{}-{}-{}.svg", type_name, state_name, style)
        }
    }
}

impl RenderOnce for WindowControl {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let icon_path = self.icon(window.is_window_active(), window.appearance());
        let icon = img(ImageSource::Resource(Resource::Embedded(icon_path.into())))
            .size_4();

        let control_type = self.control_type;
        let update_control_state = move |cx: &mut App, new_state: WindowControlState| {
            let states = cx.default_global::<ControlsState>();
            match control_type {
                WindowControlType::Minimize => {
                    states.minimize_control_state = new_state;
                },

                WindowControlType::Restore | WindowControlType::Maximize => {
                    states.maximize_or_restore_control_state = new_state;
                },

                WindowControlType::Close => {
                    states.close_control_state = new_state;
                }
            }
        };

        h_flex()
            .id(self.id)
            .group("")
            .cursor_pointer()
            .justify_center()
            .content_center()
            .rounded_2xl()
            .w(px(16.))
            .h(px(16.))
            .child(icon)
            .on_mouse_move(|_, _, cx| cx.stop_propagation())
            .on_hover(move |hover, _, cx| {
                let state = match hover {
                    true => WindowControlState::Hover,
                    false => WindowControlState::Normal
                };
                update_control_state(cx, state);
            })
            .on_mouse_down(MouseButton::Left, move |_, _,cx|{
                update_control_state(cx, WindowControlState::Active);
            })
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                update_control_state(cx, WindowControlState::Normal);

                match self.control_type {
                    WindowControlType::Minimize => window.minimize_window(),
                    WindowControlType::Restore => window.zoom_window(),
                    WindowControlType::Maximize => window.zoom_window(),
                    WindowControlType::Close => window.dispatch_action(
                        self.close_action
                            .as_ref()
                            .expect("Use WindowControl::new_close() for close control.")
                            .boxed_clone(),
                        cx,
                    ),
                }
            })
    }
}
