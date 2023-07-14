use std::ops::Deref;

use log::info;
use triangles::prelude::{Line2d, Polygon2d, PolygonPath, StaticTriangle2d, Triangle2d};
use yew::{function_component, html, use_state, use_state_eq, Callback, Html, UseStateHandle};

use crate::components::render2d::{
    CanvasMouseEvent, CssColor, CssStyle, Figure, PolygonList, Render2d,
};

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
            big_triangle.clone().to_any_polygon(),
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

    let current_selection = use_state_eq(|| None);

    let polygons: UseStateHandle<PolygonList> = use_state(|| figure_list.clone().into());
    let write_polygons = polygons.clone();
    let on_mouse_event = Callback::from(move |event: CanvasMouseEvent| {
        let x = current_selection.deref();
        let mut found = None;
        let r = event.resolution() * event.resolution() * 100.0;
        for pt in big_triangle.points() {
            if r >= pt.dist_square(&(event.x(), event.y()).into()) {
                found = Some(*pt);
            }
        }
        if x == &found {
            return;
        }
        current_selection.set(found);

        if let Some(marker_pos) = found {
            let marker = Figure::marker(CssStyle::Color(CssColor::Green), marker_pos);
            let mut new_list = figure_list.clone();
            new_list.push(marker);
            write_polygons.set(Into::<PolygonList>::into(new_list))
        } else {
            write_polygons.set(Into::<PolygonList>::into(figure_list.clone()));
        }
    });
    let p = polygons.deref().clone();
    html! {<Render2d  polygons={p} {on_mouse_event}/>}
}
