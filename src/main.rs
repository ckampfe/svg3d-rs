use nalgebra::{Isometry3, Matrix4, Perspective3, Point3, Vector3};
use std::collections::HashMap;
use svg::node::element::{Group, Polygon};
use svg::Document;

type StyleMap<V> = HashMap<String, V>;
type Face = [Point3<f32>; 3];

fn winding(face: &Face) -> f32 {
    let [p1, p2, p3] = face;
    (p2 - p1).cross(&(p3 - p1))[2]
}

fn cube() -> Vec<Face> {
    let vertices = [
        Point3::new(-0.5, -0.5, -0.5),
        Point3::new(-0.5, 0.5, -0.5),
        Point3::new(0.5, 0.5, -0.5),
        Point3::new(0.5, -0.5, -0.5),
        Point3::new(-0.5, -0.5, 0.5),
        Point3::new(-0.5, 0.5, 0.5),
        Point3::new(0.5, 0.5, 0.5),
        Point3::new(0.5, -0.5, 0.5),
    ];

    let indices = [
        [0, 3, 1],
        [1, 3, 2],
        [0, 1, 5],
        [0, 5, 4],
        [1, 2, 5],
        [6, 5, 2],
        [7, 6, 2],
        [7, 2, 3],
        [7, 3, 0],
        [4, 7, 0],
        [5, 6, 4],
        [4, 6, 7],
    ];

    indices
        .iter()
        .map(|group| [vertices[group[0]], vertices[group[1]], vertices[group[2]]])
        .collect()
}

fn octahedron() -> Vec<Face> {
    let f: f32 = 2.0f32.sqrt() / 2.0;
    let vertices = [
        Point3::new(0.0, -1.0, 0.0),
        Point3::new(-f, 0.0, f),
        Point3::new(f, 0.0, f),
        Point3::new(f, 0.0, -f),
        Point3::new(-f, 0.0, -f),
        Point3::new(0.0, 1.0, 0.0),
    ];

    let indices: [[usize; 3]; 8] = [
        [0, 2, 1],
        [0, 3, 2],
        [0, 4, 3],
        [0, 1, 4],
        [5, 1, 2],
        [5, 2, 3],
        [5, 3, 4],
        [5, 4, 1],
    ];

    indices
        .iter()
        .map(|group| [vertices[group[0]], vertices[group[1]], vertices[group[2]]])
        .collect()
}

fn icosahedron() -> Vec<Face> {
    let vertices = [
        Point3::new(0.000, 0.000, 1.000),
        Point3::new(0.894, 0.000, 0.447),
        Point3::new(0.276, 0.851, 0.447),
        Point3::new(-0.724, 0.526, 0.447),
        Point3::new(-0.724, -0.526, 0.447),
        Point3::new(0.276, -0.851, 0.447),
        Point3::new(0.724, 0.526, -0.447),
        Point3::new(-0.276, 0.851, -0.447),
        Point3::new(-0.894, 0.000, -0.447),
        Point3::new(-0.276, -0.851, -0.447),
        Point3::new(0.724, -0.526, -0.447),
        Point3::new(0.000, 0.000, -1.000),
    ];

    let indices = [
        [0, 1, 2],
        [0, 2, 3],
        [0, 3, 4],
        [0, 4, 5],
        [0, 5, 1],
        [11, 7, 6],
        [11, 8, 7],
        [11, 9, 8],
        [11, 10, 9],
        [11, 6, 10],
        [1, 6, 2],
        [2, 7, 3],
        [3, 8, 4],
        [4, 9, 5],
        [5, 10, 1],
        [6, 7, 2],
        [7, 8, 3],
        [8, 9, 4],
        [9, 10, 5],
        [10, 6, 1],
    ];

    indices
        .iter()
        .map(|group| [vertices[group[0]], vertices[group[1]], vertices[group[2]]])
        .collect()
}

struct Mesh<'a, T> {
    faces: &'a [Face],
    style: HashMap<String, String>,
    shader: Option<Box<Fn(usize, f32) -> StyleMap<T>>>,
}

impl<'a, T> Mesh<'a, T> {
    fn new(faces: &'a [Face]) -> Self {
        Mesh {
            faces,
            style: HashMap::new(),
            shader: None,
        }
    }
}

struct Camera {
    view: Isometry3<f32>,
    projection: Perspective3<f32>,
}

impl Camera {
    fn new(
        fovy: f32,
        aspect: f32,
        near: f32,
        far: f32,
        from: Point3<f32>,
        to: Point3<f32>,
        up: Vector3<f32>,
    ) -> Self {
        Camera {
            view: Isometry3::look_at_rh(&from, &to, &up),
            projection: Perspective3::new(aspect, fovy, near, far),
        }
    }
}

struct Viewport {
    minx: f32,
    miny: f32,
    width: f32,
    height: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Viewport {
            minx: -0.5,
            miny: -0.5,
            width: 1.0,
            height: 1.0,
        }
    }
}

struct Scene<'a, T> {
    meshes: &'a [Mesh<'a, T>],
}

impl<'a, T> Scene<'a, T> {
    fn new(meshes: &'a [Mesh<T>]) -> Self {
        Scene { meshes }
    }
}

struct View<'a, T> {
    camera: Camera,
    scene: Scene<'a, T>,
    viewport: Viewport,
}

impl<'a, T> View<'a, T> {
    fn new(camera: Camera, scene: Scene<'a, T>) -> Self {
        View {
            camera,
            scene,
            viewport: Viewport::default(),
        }
    }
}

struct Engine<'a, T> {
    views: &'a [View<'a, T>],
}

impl<'a, T> Engine<'a, T> {
    fn new(views: &'a [View<T>]) -> Self {
        Engine { views }
    }

    fn render(&self, filename: String) {
        let view_box = (-0.5, -0.5, 1.0, 1.0);
        let mut document = Document::new()
            .set("viewBox", view_box)
            .set("width", 512)
            .set("height", 512);

        for view in self.views {
            let projection =
                view.camera.projection.to_homogeneous() * view.camera.view.to_homogeneous();
            for mesh in view.scene.meshes {
                document = document.add(self.create_group(projection, &view.viewport, mesh));
            }
        }

        svg::save(filename, &document).unwrap();
    }

    fn create_group(&self, projection: Matrix4<f32>, viewport: &Viewport, mesh: &Mesh<T>) -> Group {
        let faces = &mesh.faces;
        // let default_style = &mesh.style;

        // from xyz to xyzw
        let with_w = faces.iter().map(|[p1, p2, p3]| {
            [
                p1.to_homogeneous(),
                p2.to_homogeneous(),
                p3.to_homogeneous(),
            ]
        });

        let projected =
            with_w.map(|[p1, p2, p3]| [projection * p1, projection * p2, projection * p3]);

        let points_by_w = projected.map(|[p1, p2, p3]| {
            [
                Point3::new(p1.x / p1.w, p1.y / p1.w, p1.z / p1.w),
                Point3::new(p2.x / p2.w, p2.y / p2.w, p2.z / p2.w),
                Point3::new(p3.x / p3.w, p3.y / p3.w, p3.z / p3.w),
            ]
        });

        let viewport_transformed: Vec<Face> = points_by_w
            .map(|mut face| {
                face.iter_mut().for_each(|point| {
                    point.x = (1.0 + point.x) * viewport.width / 2.0 + viewport.minx;
                    point.y = (1.0 - point.y) * viewport.height / 2.0 + viewport.miny;
                });

                face
            })
            .collect();

        let mut z_centroids = viewport_transformed
            .into_iter()
            .map(|face| {
                let z_centroid = face.iter().map(|point| point[2]).sum::<f32>() / 3.0;
                (face, z_centroid)
            })
            .collect::<Vec<(Face, f32)>>();

        z_centroids
            .sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut sorted_faces = z_centroids
            .into_iter()
            .map(|(face, _)| face)
            .collect::<Vec<Face>>();

        sorted_faces.reverse();

        let mut group = Group::new()
            .set("fill", "white")
            // .set("fill-opacity", 0.75)
            .set("fill-opacity", 1.0)
            .set("stroke", "black")
            .set("stroke-linejoin", "round")
            .set("stroke-width", 0.005);

        for face in sorted_faces {
            let winding = winding(&face);
            // let style = shader(1, winding);

            if winding > 0.0 {
                // there is no first-class points method, PR this maybe?
                let polygon = Polygon::new().set(
                    "points",
                    face.iter()
                        .map(|point| [point.x.to_string(), point.y.to_string()].join(","))
                        .collect::<Vec<String>>()
                        .join(" "),
                );

                group = group.add(polygon)
            }
        }

        group
    }
}

fn main() {
    let camera = Camera::new(
        15.0,
        1.0,
        10.0,
        100.0,
        Point3::new(13.0, 2.0, 20.0),
        Point3::new(0.0, 0.0, 0.0),
        Vector3::y(),
    );

    let octahedron: Vec<Face> = octahedron()
        .iter()
        .map(|face| [15.0 * face[0], 15.0 * face[1], 15.0 * face[2]])
        .collect();

    let mesh = Mesh::<String>::new(&octahedron);
    let meshes = [mesh];

    let view = View::new(camera, Scene::new(&meshes));
    let views = [view];
    let engine = Engine::new(&views);
    engine.render("octahedron.svg".to_string())
}
