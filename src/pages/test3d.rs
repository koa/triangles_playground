use std::sync::Arc;

use three_d::{
    degrees, radians, vec3, Camera, ClearState, Color, ColorMaterial, Context, CpuMesh, Gm, Mat4,
    Mesh, Object, OrbitControl, Positions, RenderTarget, Viewport,
};
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use yew::{function_component, html, Html};

use crate::components::canvas::Canvas;
use crate::components::canvas::WithRender;

#[derive(Clone, PartialEq)]
struct WebGl {}

impl WithRender for WebGl {
    fn rand(self, canvas: &HtmlCanvasElement) {
        let mut webgl2_context: WebGl2RenderingContext = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        let glow_context = glow::Context::from_webgl2_context(webgl2_context);

        // Create a camera
        let mut camera = Camera::new_perspective(
            Viewport::new_at_origo(1, 1),
            vec3(0.0, 0.0, 2.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            10.0,
        );

        // Create a CPU-side mesh consisting of a single colored triangle
        let positions = vec![
            vec3(0.5, -0.5, 0.0),  // bottom right
            vec3(-0.5, -0.5, 0.0), // bottom left
            vec3(0.0, 0.5, 0.0),   // top
        ];
        let colors = vec![
            Color::RED,   // bottom right
            Color::GREEN, // bottom left
            Color::BLUE,  // top
        ];
        let cpu_mesh = CpuMesh {
            positions: Positions::F32(positions),
            colors: Some(colors),
            ..Default::default()
        };
        let context = Context::from_gl_context(Arc::new(glow_context)).unwrap();
        let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

        // Construct a model, with a default color material, thereby transferring the mesh data to the GPU
        let mut model = Gm::new(Mesh::new(&context, &cpu_mesh), ColorMaterial::default());

        // Add an animation to the triangle.
        model.set_animation(|time| Mat4::from_angle_y(radians(time * 0.005)));
        camera.set_viewport(Viewport {
            x: canvas.offset_left(),
            y: canvas.offset_top(),
            width: canvas.width(),
            height: canvas.height(),
        });

        RenderTarget::screen(&context, canvas.width(), canvas.height())
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(&camera, &model, &[]);
    }
}

#[function_component(Test3d)]
pub fn test_3d() -> Html {
    html!(
            <Canvas<WebGl2RenderingContext, WebGl>
                //send props when create a Render
                render={Box::new(WebGl{})}>
                {"The browser is not supported."}
            </Canvas<WebGl2RenderingContext, WebGl >>
    )
}
