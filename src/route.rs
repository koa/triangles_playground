use yew::{html, Html};
use yew_nested_router::Target;

use crate::pages::test3d::Test3d;
use crate::pages::triangle_cut_2d::TriangleCut2d;

#[derive(Debug, Default, Clone, PartialEq, Eq, Target)]
pub enum AppRoute {
    #[default]
    Basic2d,
    Test3d,
}

pub fn switch_main(switch: AppRoute) -> Html {
    match switch {
        AppRoute::Basic2d => html! {<TriangleCut2d/>},
        AppRoute::Test3d => {
            html! {<Test3d/>}
        }
    }
}
