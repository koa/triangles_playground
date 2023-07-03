use log::info;
use patternfly_yew::prelude::{
    BackdropViewer, Nav, NavItem, NavRouterItem, Page, PageSidebar, ToastViewer,
};
use yew::function_component;
use yew::html_nested;
use yew::{html, Callback, Html, MouseEvent};
use yew_nested_router::Router;
use yew_nested_router::Switch as RouterSwitch;

use crate::route::switch_main;
use crate::route::AppRoute;

pub mod basic2d;

#[function_component(MainPage)]
pub fn main_page() -> Html {
    info!("Hello");
    html! {
        <BackdropViewer>
            <ToastViewer>
                <Router<AppRoute>>
                    <Page sidebar={html_nested! {<PageSidebar><Sidebar/></PageSidebar>}}>
                      //<ToastViewer/>
                      //logo={logo}
                        <RouterSwitch<AppRoute>
                            render = { switch_main}
                        />
                    </Page>
                </Router<AppRoute>>
            </ToastViewer>
        </BackdropViewer>
    }
}

#[function_component(Sidebar)]
fn authenticated_sidebar() -> Html {
    let logout = Callback::from(move |_: MouseEvent| {});
    html! {
        <Nav>
            <NavRouterItem<AppRoute> to={AppRoute::Home}>{"Start"}</NavRouterItem<AppRoute>>
            <span onclick={logout}><NavItem>{"Logout"}</NavItem></span>
        </Nav>
    }
}
