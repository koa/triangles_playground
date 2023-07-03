use log::info;
use triangles::prelude::{Polygon2d, StaticTriangle2d};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::function_component;
use yew::{html, use_state, Callback, Html};

use crate::components::canvas2d::Canvas;
use crate::components::canvas2d::WithRender;
use crate::components::render2d::{PolygonList, Render2d};

//Befor impl WithRander, derive Clone and PartialEq first!
#[derive(Clone, PartialEq)]
struct Render {
    //use this struct send props to canvas
    sakara: usize,
}

impl WithRender for Render {
    fn rand(self, canvas: &HtmlCanvasElement) {
        let interface: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        let width = canvas.width() as f64;
        let height = canvas.height() as f64;
        interface.clear_rect(0.0, 0.0, width, height);
        interface.set_fill_style(&JsValue::from_str("#fe5c5a"));
        interface.set_font("100px sans-serif");
        interface.set_text_baseline("top");

        let sakara = (vec!['🐟'; self.sakara]).into_iter().collect::<String>();
        let text = &format!("{}🐟{width};{height}🐟{}", sakara, sakara);

        let text_metrics = interface.measure_text(text).unwrap();
        let (actual_bounding_box_descent, font_bounding_box_descent, width) = (
            text_metrics.actual_bounding_box_descent(),
            text_metrics.font_bounding_box_descent(),
            text_metrics.width(),
        );
        info!(
            "a:{} b:{} c:{}",
            actual_bounding_box_descent, font_bounding_box_descent, width
        );

        let text_pos = (100.0, 100.0);

        interface.fill_text(text, text_pos.0, text_pos.1).unwrap();
        interface.set_stroke_style(&JsValue::from_str("red"));
        interface.stroke_rect(text_pos.0, text_pos.1, width, actual_bounding_box_descent);

        interface.set_stroke_style(&JsValue::from_str("green"));
        interface.stroke_rect(text_pos.0, text_pos.1, width, font_bounding_box_descent)
    }
}
#[function_component(Basic2d)]
pub fn basic_2d() -> Html {
    let big_triangle = StaticTriangle2d::new(
        (-100.0, -50.0).into(),
        (100.0, -50.0).into(),
        (0.0, 50.0).into(),
    )
    .to_any_polygon();
    let polygons: PolygonList = vec![big_triangle].into();
    html! {<Render2d {polygons}/>}
}

#[function_component(OldBasic2d)]
pub fn old_basic2d() -> Html {
    let sakara_state = use_state(|| 0);

    let onclick = {
        let sakara_state = sakara_state.clone();
        Callback::from(move |_| sakara_state.set(*sakara_state + 1))
    };
    html!(
        <>
            <button {onclick}>{"+🐟"}</button>
            <Canvas<CanvasRenderingContext2d, Render>
                //Just use style, canvas can suit automatically.
                style="
                    width: 100vw;
                    height: calc(100vh - 32px);
                "
                //send props when create a Render
                rander={Box::new(Render{sakara: *sakara_state})}
            >
                {"The browser is not supported."}
            </Canvas<CanvasRenderingContext2d, Render >>
        </>
    )
}
