use log::info;
use triangles::prelude::{Line2d, PolygonPath, Triangle2d};
use triangles::prelude::{Polygon2d, StaticTriangle2d};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::function_component;
use yew::{html, use_state, Callback, Html};

use crate::components::canvas2d::Canvas;
use crate::components::canvas2d::WithRender;
use crate::components::render2d::{CssColor, CssStyle, Figure, PolygonList, Render2d};

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
    );
    let small_triangle = StaticTriangle2d::new(
        (-50.0, 25.0).into(),
        (00.0, -25.0).into(),
        (50.0, 25.0).into(),
    );
    let cut_polygon = &small_triangle;
    let path = big_triangle.cut(cut_polygon);

    let mut figure_list = vec![
        Figure::polygon(
            CssStyle::Color(CssColor::Blue),
            big_triangle.to_any_polygon(),
        ),
        Figure::polygon(
            CssStyle::Color(CssColor::Green),
            small_triangle.to_any_polygon(),
        ),
    ];
    //figure_list.clear();
    match &path {
        PolygonPath::Enclosed => {
            figure_list.push(Figure::polygon(
                CssStyle::Color(CssColor::Red),
                cut_polygon.to_any_polygon(),
            ));
        }
        PolygonPath::CutSegments(segments) => {
            for segment in segments {
                let mut points = Vec::new();
                let start_cut = segment.start_cut();
                let end_cut = segment.end_cut();
                if let (Some(start_line), Some(end_line)) = (
                    cut_polygon.lines().nth(start_cut.start_pt_idx()),
                    cut_polygon.lines().nth(end_cut.start_pt_idx()),
                ) {
                    points.push(start_line.pt_along(start_cut.polygon_pos()));
                    for p in cut_polygon.points_of_range(segment.copy_points()) {
                        points.push(*p);
                    }
                    points.push(end_line.pt_along(end_cut.polygon_pos()));
                    figure_list.push(Figure::lines(CssStyle::Color(CssColor::Red), points));
                }
            }
        }
        PolygonPath::None => {}
    }

    let polygons: PolygonList = figure_list.into();
    let on_mouse_event = Callback::from(|event| info!("Event: {event:?}"));
    html! {<Render2d {polygons} {on_mouse_event}/>}
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
                render={Box::new(Render{sakara: *sakara_state})}
            >
                {"The browser is not supported."}
            </Canvas<CanvasRenderingContext2d, Render >>
        </>
    )
}
