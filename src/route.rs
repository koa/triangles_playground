use yew::{html, Html};
use yew_nested_router::Target;

use crate::pages::basic2d::Basic2d;

#[derive(Debug, Default, Clone, PartialEq, Eq, Target)]
pub enum AppRoute {
    #[default]
    Home,
}

pub fn switch_main(switch: AppRoute) -> Html {
    match switch {
        AppRoute::Home => html! {<Basic2d/>},
    }
}
