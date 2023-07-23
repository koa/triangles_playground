use std::ops::Deref;
use std::rc::Rc;

use num_traits::Zero;
use triangles::prelude::{Number, Point2d, Polygon2d, StaticPoint2d, StaticTriangle2d, Triangle2d};
use yew::{function_component, html, use_state, Callback, Html, UseStateHandle};

use crate::components::render2d::{
    CanvasMouseEvent, CssColor, CssStyle, Figure, PolygonList, Render2d,
};

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

#[function_component(TriangleCut2d)]
pub fn triangle_cut_2d() -> Html {
    let big_triangle_state = use_state(|| {
        StaticTriangle2d::new(
            (-100.0, 0.0).into(),
            (100.0, 0.0).into(),
            (0.0, 100.0).into(),
        )
    });
    let big_triangle = *big_triangle_state;
    let small_triangle = Rc::new(StaticTriangle2d::<StaticPoint2d>::new(
        (-50.0, 25.0).into(),
        (00.0, -25.0).into(),
        (50.0, 25.0).into(),
    ));
    let generate_cutting_triangles = enclose! {(big_triangle_state) move || {
            let cut_polygon = small_triangle.clone();
            let path = big_triangle.cut(cut_polygon.deref());

            let mut figure_list = vec![
                /*
                Figure::polygon(
                    CssStyle::Color(CssColor::Blue),
                    big_triangle_state.clone().to_any_polygon(),
                ),
                Figure::polygon(
                    CssStyle::Color(CssColor::Green),
                    small_triangle.to_any_polygon(),
                ),*/
            ];
            for pt in big_triangle_state.points(){
                figure_list.push(Figure::marker(CssStyle::Color(CssColor::Blue),*pt));
            }
            //figure_list.clear();

            let triangles = big_triangle.cut_to_triangles(cut_polygon.as_ref());;
            for (triangles,style) in triangles.iter().zip([CssStyle::Color(CssColor::Green),CssStyle::Color(CssColor::Red)].iter()) {
                for triangle in triangles{
                    let triangle=triangle.coordinates_triangle();
                    figure_list.push(Figure::polygon(style.clone(), triangle.to_any_polygon()));
                }
            }
    /*
        let polygons =  big_triangle.compose_cut_polygons(cut_polygon.as_ref(), &path);
        for (polygons,style) in polygons.iter().zip([CssStyle::Color(CssColor::Green),CssStyle::Color(CssColor::Red)].iter()) {
                for polygon in polygons{
                figure_list.push(Figure::polygon(style.clone(),polygon.clone().to_any_polygon()));
                    }
        }*/

            figure_list
        }};

    let current_selection = use_state(|| None);

    let polygons: UseStateHandle<PolygonList> = use_state(|| generate_cutting_triangles().into());
    let update_polygons = enclose! {(polygons)
    move | found_idx: Option<usize>, big_triangle: &StaticTriangle2d<StaticPoint2d>| {
        let figure_list = generate_cutting_triangles();
        let found = found_idx.and_then(|idx| big_triangle.get_point(idx));

        if let Some(marker_pos) = found {
            let marker = Figure::marker(CssStyle::Color(CssColor::Green), *marker_pos);
            let mut new_list = figure_list.clone();
            new_list.push(marker);
            polygons.set(Into::<PolygonList>::into(new_list))
        } else {
            polygons.set(Into::<PolygonList>::into(figure_list.clone()));
        }
    }};
    let on_mouse_event = Callback::from(
        enclose! {(big_triangle_state) move |event: CanvasMouseEvent| {
            let last_selection = *current_selection.deref();
                    let mouse_pt = (event.x(), event.y()).into();
            if (event.buttons() & 1) != 0 {
                if let Some(selected_corner) = last_selection {
                    let original_triangle = *big_triangle_state;
                    let p1 = if selected_corner != 0 {
                        original_triangle.p1()
                    } else {
                        &mouse_pt
                    };
                    let p2 = if selected_corner != 1 {
                        original_triangle.p2()
                    } else {
                        &mouse_pt
                    };
                    let p3 = if selected_corner != 2 {
                        original_triangle.p3()
                    } else {
                        &mouse_pt
                    };
                    let moved_triangle = StaticTriangle2d::new(*p1,*p2,*p3);
                    if moved_triangle.area()>Number::zero(){

                        update_polygons(last_selection, big_triangle_state.deref());
                        big_triangle_state.set( moved_triangle);
                    };
                    return ;
                }
            }
            let mut found_idx = None;
            let r = event.resolution() * event.resolution() * 100.0;
            for (idx, pt) in big_triangle.points().enumerate() {
                if r >= pt.dist_square(&mouse_pt) {
                    found_idx = Some(idx);
                }
              }
            if last_selection != found_idx {
                current_selection.set(found_idx);
                update_polygons(found_idx, big_triangle_state.deref());
            }
        }},
    );
    let p = polygons.deref().clone();
    html! {<Render2d  polygons={p} {on_mouse_event}/>}
}
