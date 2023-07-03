use std::default::Default;

use log::info;
use num_traits::Pow;
use triangles::prelude::{
    AnyPolygon, BoundingBox, BoundingBoxValues, Number, Point2d, Polygon2d, StaticTriangle2d,
};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::html::IntoPropValue;
use yew::Properties;
use yew::{function_component, html, use_state, Callback, Html};

use crate::components::canvas2d::Canvas;
use crate::components::canvas2d::WithRender;
use crate::components::render2d::_RenderProperties::polygons;

//Befor impl WithRander, derive Clone and PartialEq first!
#[derive(Clone, PartialEq)]
struct Render {
    polygons: PolygonList,
}
struct ScreenProject2d {
    scale: Number,
    x_offset: Number,
    y_offset: Number,
}

impl ScreenProject2d {
    fn from_bounding_box(bbox: &BoundingBoxValues, canvas_width: f64, canvas_height: f64) -> Self {
        let canvas_width = Into::<Number>::into(canvas_width);
        let canvas_height = Into::<Number>::into(canvas_height);
        info!("canvas: {canvas_width},{canvas_height}");
        let fact = Number::min(canvas_width / bbox.width(), canvas_height / bbox.height());
        let x_offset = -bbox.min_x() * fact + ((canvas_width - bbox.width() * fact) / 2.0);

        let y_offset = -bbox.max_y() * -fact + ((canvas_height - bbox.height() * fact) / 2.0);
        Self {
            scale: fact,
            x_offset,
            y_offset,
        }
    }
    fn project_point(&self, p: &Point2d) -> (f64, f64) {
        (
            (self.scale * p.x() + self.x_offset).into(),
            (-self.scale * p.y() + self.y_offset).into(),
        )
    }

    pub fn scale(&self) -> Number {
        self.scale
    }
}
#[cfg(test)]
mod test {
    use triangles::prelude::BoundingBoxValues;

    use crate::components::render2d::ScreenProject2d;

    #[test]
    fn test_projection() {
        let b = BoundingBoxValues::new((-10.0).into(), (-30.0).into(), 100.0.into(), 150.0.into());
        let p = ScreenProject2d::from_bounding_box(&b, 65.0, 90.0);
        let (x, y) = p.project_point(&(42.0, 23.0).into());
        assert_eq!(x, 31.0);
        assert_eq!(y, 63.5);
    }
}
enum TickSideHorizontal {
    Left,
    Right,
}
enum TickSideVertical {
    Top,
    Bottom,
}
impl WithRender for Render {
    fn rand(self, canvas: &HtmlCanvasElement) {
        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        let width = canvas.width() as f64;
        let height = canvas.height() as f64;
        ctx.clear_rect(0.0, 0.0, width, height);

        let mut bbox: BoundingBox = BoundingBox::default();
        for polygon in &self.polygons.0 {
            for p in polygon.points() {
                bbox += *p;
            }
        }
        match bbox {
            BoundingBox::Empty => {}
            BoundingBox::Box(bbox) => {
                let bbox = bbox.expand(0.1.into());
                let p = ScreenProject2d::from_bounding_box(&bbox, width, height);
                let (zero_x, zero_y) = p.project_point(&(0.0, 0.0).into());
                let (min_x, min_y) = p.project_point(&(bbox.min_x(), bbox.min_y()).into());
                let (max_x, max_y) = p.project_point(&(bbox.max_x(), bbox.max_y()).into());
                let (tick_y, tick_side_vertical) = if zero_y < 0.0 {
                    (0.0, TickSideVertical::Bottom)
                } else if zero_y > height {
                    (height, TickSideVertical::Top)
                } else {
                    ctx.begin_path();
                    ctx.move_to(min_x, zero_y);
                    ctx.line_to(max_x, zero_y);
                    ctx.stroke();
                    (
                        zero_y,
                        if zero_y > height / 2.0 {
                            TickSideVertical::Top
                        } else {
                            TickSideVertical::Bottom
                        },
                    )
                };

                let (tick_x, tick_side_horizontal) = if zero_x < 0.0 {
                    (0.0, TickSideHorizontal::Right)
                } else if zero_x > width {
                    (0.0, TickSideHorizontal::Left)
                } else {
                    ctx.begin_path();
                    ctx.move_to(zero_x, min_y);
                    ctx.line_to(zero_x, max_y);
                    ctx.stroke();
                    (
                        zero_x,
                        if zero_x > width / 2.0 {
                            TickSideHorizontal::Left
                        } else {
                            TickSideHorizontal::Right
                        },
                    )
                };

                let y_step = find_optimal_step(40.0 / p.scale().0);
                let mut y_tick = y_step;
                while y_tick < bbox.max_y().0 {
                    Self::draw_y_tick(&ctx, &p, &tick_side_horizontal, y_tick, tick_x);
                    y_tick += y_step;
                }
                y_tick = -y_step;
                while y_tick > bbox.min_y().0 {
                    Self::draw_y_tick(&ctx, &p, &tick_side_horizontal, y_tick, tick_x);
                    y_tick -= y_step;
                }

                let x_step = find_optimal_step(40.0 / p.scale().0);
                let mut x_tick = x_step;
                while x_tick < bbox.max_x().0 {
                    Self::draw_x_tick(&ctx, &p, &tick_side_vertical, x_tick, tick_y);
                    x_tick += x_step;
                }
                x_tick = -x_step;
                while x_tick > bbox.min_x().0 {
                    Self::draw_x_tick(&ctx, &p, &tick_side_vertical, x_tick, tick_y);
                    x_tick -= x_step;
                }

                for polygon in &self.polygons.0 {
                    let mut iter = polygon.points();
                    if let Some(start_pt) = iter.next() {
                        let (x, y) = p.project_point(start_pt);
                        ctx.begin_path();
                        ctx.move_to(x, y);
                        for next_pt in iter {
                            let (x, y) = p.project_point(next_pt);
                            ctx.line_to(x, y);
                        }
                        ctx.close_path();
                        ctx.stroke();
                    }
                }
            }
        }
    }
}

impl Render {
    fn draw_x_tick(
        ctx: &CanvasRenderingContext2d,
        p: &ScreenProject2d,
        tick_side_vertical: &TickSideVertical,
        x_tick: f64,
        tick_y: f64,
    ) {
        let (x, _) = p.project_point(&(x_tick, 0.0).into());
        let label = &format!("{}", x_tick);
        let text_metrics = ctx.measure_text(label).unwrap();
        let text_width = text_metrics.width();
        match tick_side_vertical {
            TickSideVertical::Top => {
                ctx.begin_path();
                ctx.move_to(x, tick_y);
                ctx.line_to(x, tick_y - 5.0);
                ctx.stroke();
                ctx.fill_text(label, x - text_width / 2.0, tick_y - 10.0)
                    .unwrap();
            }
            TickSideVertical::Bottom => {
                ctx.begin_path();
                ctx.move_to(x, tick_y);
                ctx.line_to(x, tick_y + 5.0);
                ctx.stroke();
                ctx.fill_text(label, x - text_width / 2.0, tick_y + 20.0)
                    .unwrap();
            }
        }
    }
    fn draw_y_tick(
        ctx: &CanvasRenderingContext2d,
        p: &ScreenProject2d,
        tick_side_horizontal: &TickSideHorizontal,
        y_tick: f64,
        tick_x: f64,
    ) {
        let (_, y) = p.project_point(&(0.0, y_tick).into());
        let label = &format!("{}", y_tick);
        match tick_side_horizontal {
            TickSideHorizontal::Left => {
                ctx.begin_path();
                ctx.move_to(tick_x, y);
                ctx.line_to(tick_x + 5.0, y);
                ctx.stroke();
                ctx.fill_text(label, tick_x + 10.0, y).unwrap();
            }
            TickSideHorizontal::Right => {
                let text_metrics = ctx.measure_text(label).unwrap();
                ctx.begin_path();
                ctx.move_to(tick_x, y);
                ctx.line_to(tick_x - 5.0, y);
                ctx.stroke();
                ctx.fill_text(label, tick_x - 10.0 - text_metrics.width(), y)
                    .unwrap();
            }
        }
    }
}

fn find_optimal_step(step: f64) -> f64 {
    let log10 = step.log10();
    let fract = (log10 - log10.floor());
    let (pow, scale) = if fract < 0.17 {
        (log10.floor(), 1.0)
    } else if fract < 0.5 {
        (log10.floor(), 2.0)
    } else if fract < 0.85 {
        (log10.floor(), 5.0)
    } else {
        (log10.ceil(), 1.0)
    };
    10.0.pow(pow) * scale
}

#[derive(Clone, PartialEq)]
pub struct PolygonList(Vec<AnyPolygon>);

impl IntoPropValue<Vec<AnyPolygon>> for PolygonList {
    fn into_prop_value(self) -> Vec<AnyPolygon> {
        self.0
    }
}

impl From<Vec<AnyPolygon>> for PolygonList {
    fn from(value: Vec<AnyPolygon>) -> Self {
        PolygonList(value)
    }
}

#[derive(Properties, PartialEq)]
pub struct RenderProperties {
    pub polygons: PolygonList,
}

#[function_component(Render2d)]
pub fn render_2d(properties: &RenderProperties) -> Html {
    html!(
            <Canvas<CanvasRenderingContext2d, Render>
                //send props when create a Render
                rander={Box::new(Render{polygons:properties.polygons.clone()})}
            >
                {"The browser is not supported."}
            </Canvas<CanvasRenderingContext2d, Render >>
    )
}
