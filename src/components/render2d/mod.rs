use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::default::Default;
use std::f64::consts::PI;
use std::ops::Deref;
use std::rc::Rc;

use log::info;
use num_traits::{One, Pow};
use triangles::prelude::{
    AnyPolygon, BoundingBox, BoundingBoxValues, Float, Number, Point2d, Polygon2d,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent};
use yew::html::IntoPropValue;
use yew::{function_component, html, Html};
use yew::{Callback, Properties};

use crate::components::canvas2d::Canvas;
use crate::components::canvas2d::WithRender;
use crate::components::render2d::tick_sequence::TickSequence;
use crate::components::render2d::CssStyle::Color;

//Befor impl WithRander, derive Clone and PartialEq first!
#[derive(Clone)]
struct Render {
    display_list: Rc<[Figure]>,
    last_projection: Rc<RefCell<Option<ScreenProject2d>>>,
}

impl PartialEq for Render {
    fn eq(&self, other: &Self) -> bool {
        self.display_list == other.display_list
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
struct ScreenProject2d {
    scale: Number,
    x_offset: Number,
    y_offset: Number,
}

impl ScreenProject2d {
    fn from_bounding_box(bbox: &BoundingBoxValues, canvas_width: f64, canvas_height: f64) -> Self {
        let canvas_width = Into::<Number>::into(canvas_width);
        let canvas_height = Into::<Number>::into(canvas_height);
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
    fn find_origin_point(&self, x: i32, y: i32) -> (Number, Number) {
        (
            ((Number::from(x as f64) - self.x_offset) / self.scale),
            ((Number::from(y as f64) - self.y_offset) / -self.scale),
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
        let (x, y) = p.find_origin_point(x.round() as i32, y.round() as i32);
        println!("{x},{y}");
    }
}
#[derive(Debug)]
enum TickSideHorizontal {
    Left,
    Right,
}
#[derive(Debug)]
enum TickSideVertical {
    Top,
    Bottom,
}
impl WithRender for Render {
    fn rand(self, canvas: &HtmlCanvasElement) {
        let mut ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        ctx.set_stroke_style(&CssColor::Black.value());

        let width = canvas.width() as f64;
        let height = canvas.height() as f64;
        ctx.clear_rect(0.0, 0.0, width, height);

        let mut bbox: BoundingBox = BoundingBox::default();

        for figure in self.display_list.iter() {
            bbox += figure.bbox();
        }
        match bbox {
            BoundingBox::Empty => {}
            BoundingBox::Box(bbox) => {
                let bbox = bbox.expand(0.1.into());
                let p = ScreenProject2d::from_bounding_box(&bbox, width, height);
                //let _ = self.last_projection.borrow_mut().insert(p);
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
                    (min_x, TickSideHorizontal::Right)
                } else if zero_x > width {
                    (max_x, TickSideHorizontal::Left)
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
                for y_tick in TickSequence::new(bbox.min_y().0, bbox.max_y().0, y_step).iter() {
                    Self::draw_y_tick(&ctx, &p, &tick_side_horizontal, y_tick, tick_x);
                }

                let x_step = find_optimal_step(40.0 / p.scale().0);
                for x_tick in TickSequence::new(bbox.min_x().0, bbox.max_x().0, x_step).iter() {
                    Self::draw_x_tick(&ctx, &p, &tick_side_vertical, x_tick, tick_y);
                }

                for figure in self.display_list.iter() {
                    figure.draw(&mut ctx, &p);
                }
            }
        }
    }
}

mod tick_sequence;

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
            TickSideHorizontal::Right => {
                ctx.begin_path();
                ctx.move_to(tick_x, y);
                ctx.line_to(tick_x + 5.0, y);
                ctx.stroke();
                ctx.fill_text(label, tick_x + 10.0, y).unwrap();
            }
            TickSideHorizontal::Left => {
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
    let floor = log10.floor();
    let fract = log10 - floor;
    let scale = if fract < 0.17 {
        1.0
    } else if fract < 0.5 {
        2.0
    } else if fract < 0.85 {
        5.0
    } else {
        10.0
    };
    10.0.pow(floor) * scale
}
#[derive(Clone, PartialEq, Debug)]
pub struct Figure {
    style: CssStyle,
    geometry: AnyGeometry,
}

impl Figure {
    pub(crate) fn marker(style: CssStyle, pt: Point2d) -> Figure {
        Self {
            style,
            geometry: AnyGeometry::HoverMarker(pt),
        }
    }
}

impl Figure {
    pub fn polygon(style: CssStyle, polygon: AnyPolygon) -> Self {
        Self {
            style,
            geometry: AnyGeometry::Polygon(polygon),
        }
    }
    pub fn lines(style: CssStyle, lines: Vec<Point2d>) -> Self {
        Self {
            style,
            geometry: AnyGeometry::Lines(lines),
        }
    }
    fn bbox(&self) -> BoundingBox {
        self.geometry.bounding_box()
    }
    fn draw(&self, ctx: &mut CanvasRenderingContext2d, p: &ScreenProject2d) {
        match &self.geometry {
            AnyGeometry::Polygon(polygon) => {
                let mut iter = polygon.points();
                if let Some(start_pt) = iter.next() {
                    let (x, y) = p.project_point(start_pt);
                    ctx.begin_path();
                    ctx.set_stroke_style(&self.style.value());
                    ctx.move_to(x, y);
                    for next_pt in iter {
                        let (x, y) = p.project_point(next_pt);
                        ctx.line_to(x, y);
                    }
                    ctx.close_path();
                    ctx.stroke();
                }
            }
            AnyGeometry::Lines(lines) => {
                let mut iter = lines.points();
                if let Some(start_pt) = iter.next() {
                    let (x, y) = p.project_point(start_pt);
                    ctx.begin_path();
                    ctx.set_stroke_style(&self.style.value());
                    ctx.move_to(x, y);
                    for next_pt in iter {
                        let (x, y) = p.project_point(next_pt);
                        ctx.line_to(x, y);
                    }
                    ctx.stroke();
                }
            }
            AnyGeometry::HoverMarker(pt) => {
                let (x, y) = p.project_point(pt);
                ctx.begin_path();
                ctx.set_stroke_style(&self.style.value());
                ctx.set_fill_style(&self.style.value());
                ctx.arc(x, y, 5.0, 0.0, PI * 2.0).expect("Infallible");
                ctx.fill();
                ctx.stroke();
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum CssStyle {
    Color(CssColor),
}

impl CssStyle {
    fn value(&self) -> JsValue {
        match self {
            Color(c) => c.value(),
        }
    }
}

impl Default for CssStyle {
    fn default() -> Self {
        Color(CssColor::Black)
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub enum CssColor {
    #[default]
    Black,
    Blue,
    Green,
    Red,
}

impl CssColor {
    fn value(&self) -> JsValue {
        match self {
            CssColor::Black => JsValue::from_str("black"),
            CssColor::Blue => JsValue::from_str("blue"),
            CssColor::Green => JsValue::from_str("green"),
            CssColor::Red => JsValue::from_str("red"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum AnyGeometry {
    Polygon(AnyPolygon),
    Lines(Vec<Point2d>),
    HoverMarker(Point2d),
}

impl AnyGeometry {
    fn bounding_box(&self) -> BoundingBox {
        let mut bbox = BoundingBox::default();
        match self {
            AnyGeometry::Polygon(p) => {
                for p in p.points() {
                    bbox += *p;
                }
            }
            AnyGeometry::Lines(l) => {
                for p in l {
                    bbox += *p;
                }
            }
            AnyGeometry::HoverMarker(p) => {
                bbox += *p;
            }
        }
        bbox
    }
}

#[derive(Clone, PartialEq)]
pub struct PolygonList(Rc<[Figure]>);

impl PolygonList {}

impl IntoPropValue<Rc<[Figure]>> for PolygonList {
    fn into_prop_value(self) -> Rc<[Figure]> {
        self.0
    }
}

impl From<Rc<[Figure]>> for PolygonList {
    fn from(value: Rc<[Figure]>) -> Self {
        PolygonList(value)
    }
}
impl From<Vec<Figure>> for PolygonList {
    fn from(value: Vec<Figure>) -> Self {
        PolygonList(Rc::from(value.as_slice()))
    }
}

#[derive(Properties, PartialEq)]
pub struct RenderProperties {
    pub polygons: PolygonList,
    pub on_mouse_event: Option<Callback<CanvasMouseEvent>>,
}
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CanvasMouseEvent {
    x: Number,
    y: Number,
    buttons: u16,
    resolution: Number,
}

impl CanvasMouseEvent {
    #[inline]
    pub fn x(&self) -> Number {
        self.x
    }
    #[inline]
    pub fn y(&self) -> Number {
        self.y
    }
    #[inline]
    pub fn buttons(&self) -> u16 {
        self.buttons
    }
    #[inline]
    pub fn resolution(&self) -> Number {
        self.resolution
    }
}

#[function_component(Render2d)]
pub fn render_2d(properties: &RenderProperties) -> Html {
    let last_projection = Rc::new(RefCell::new(None::<ScreenProject2d>));
    let current_projection = last_projection.clone();
    let onmouse = properties.on_mouse_event.clone().map(|mouse_callback| {
        Callback::from(move |mouse_event: MouseEvent| {
            if let Some(p) = current_projection.borrow().deref() {
                let (x, y) = p.find_origin_point(mouse_event.offset_x(), mouse_event.offset_y());

                let buttons = mouse_event.buttons();
                mouse_callback.emit(CanvasMouseEvent {
                    x,
                    y,
                    buttons,
                    resolution: Number::one() / p.scale,
                })
            }
        })
    });

    html!(
            <Canvas<CanvasRenderingContext2d, Render>
                {onmouse}
                //send props when create a Render
                render={Box::new(Render{display_list:properties.polygons.0.clone(), last_projection})}
            >
                {"The browser is not supported."}
            </Canvas<CanvasRenderingContext2d, Render >>
    )
}
