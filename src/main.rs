use nalgebra::{Isometry3, Matrix4, Perspective3, Point3, Vector3};
use std::collections::HashMap;
use svg::node::element::{Group, Polygon};
use svg::Document;
use memmap::MmapOptions;

type StyleMap<V> = HashMap<String, V>;
type Faces = Vec<Vec<Point3<f32>>>;

struct Mesh<T> {
    faces: Faces,
    style: HashMap<String, String>,
    shader: Option<Box<Fn(usize, f32) -> StyleMap<T>>>,
}

impl<T> Mesh<T> {
    fn new(faces: Faces) -> Self {
        Mesh {
            faces,
            style: HashMap::new(),
            shader: None,
        }
    }
}

fn octahedron() -> Vec<Vec<Point3<f32>>> {
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
        .map(|group| group.iter().map(|index| vertices[*index]).collect())
        .collect()
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

struct Scene<T> {
    meshes: Vec<Mesh<T>>,
}

impl<T> Scene<T> {
    fn new(meshes: Vec<Mesh<T>>) -> Self {
        Scene { meshes }
    }
}

struct View<T> {
    camera: Camera,
    scene: Scene<T>,
    viewport: Viewport,
}

impl<T> View<T> {
    fn new(camera: Camera, scene: Scene<T>) -> Self {
        View {
            camera,
            scene,
            viewport: Viewport::default(),
        }
    }
}

struct Engine<T> {
    views: Vec<View<T>>,
}

impl<T> Engine<T> {
    fn new(views: Vec<View<T>>) -> Self {
        Engine { views }
    }

    fn render(&self, filename: String) {
        let view_box = (-0.5, -0.5, 1.0, 1.0);
        let mut document = Document::new()
            .set("viewBox", view_box)
            .set("width", 512)
            .set("height", 512);

        for view in &self.views {
            let projection =
                view.camera.projection.to_homogeneous() * view.camera.view.to_homogeneous();
            for mesh in &view.scene.meshes {
                document = document.add(self.create_group(projection, &view.viewport, mesh));
            }
        }

        svg::save(filename, &document).unwrap();
    }

    fn create_group(&self, projection: Matrix4<f32>, viewport: &Viewport, mesh: &Mesh<T>) -> Group {
        let faces = &mesh.faces;
        // let default_style = &mesh.style;

        let with_w = faces.iter().map(|face| {
            vec![
                face[0].to_homogeneous(),
                face[1].to_homogeneous(),
                face[2].to_homogeneous(),
            ]
        });

        let projected = with_w.map(|face| {
            vec![
                projection * face[0],
                projection * face[1],
                projection * face[2],
            ]
        });

        let points_by_w = projected.map(|face| {
            let [p1, p2, p3] = [face[0], face[1], face[2]];

            vec![
                Point3::new(p1[0] / p1[3], p1[1] / p1[3], p1[2] / p1[3]),
                Point3::new(p2[0] / p2[3], p2[1] / p2[3], p2[2] / p2[3]),
                Point3::new(p3[0] / p3[3], p3[1] / p3[3], p3[2] / p3[3]),
            ]
        });

        let viewport_transformed: Vec<Vec<Point3<f32>>> = points_by_w
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
            .collect::<Vec<(Vec<Point3<f32>>, f32)>>();

        z_centroids
            .sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Less));

        let mut sorted_faces = z_centroids
            .into_iter()
            .map(|(face, _)| face)
            .collect::<Vec<Vec<Point3<f32>>>>();

        sorted_faces.reverse();

        let mut group = Group::new()
            .set("fill", "white")
            .set("fill-opacity", 0.75)
            .set("stroke", "black")
            .set("stroke-linejoin", "round")
            .set("stroke-width", 0.0005);

        for face in sorted_faces {
            let [p0, p1, p2] = [face[0], face[1], face[2]];
            let winding = (p1 - p0).cross(&(p2 - p0))[2];
            // let style = shader(1, winding);

            // there is no first-class points method, PR this maybe?
            let polygon = Polygon::new().set(
                "points",
                [
                    [face[0].x.to_string(), face[0].y.to_string()].join(","),
                    [face[1].x.to_string(), face[1].y.to_string()].join(","),
                    [face[2].x.to_string(), face[2].y.to_string()].join(","),
                ]
                .join(" "),
            );

            group = group.add(polygon)
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
        Point3::new(-0.2, -2.0, -0.5),
        Point3::new(0.0, 0.0, 0.0),
        Vector3::y(),
    );

    let file = std::fs::File::open("/Users/clark/code/Moon.stl").unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let (_, stl) = nom_stl::parse_stl(&mmap).unwrap();
    let faces: Faces = stl.triangles.iter().map(|triangle: &nom_stl::IndexedTriangle| {
        let v1 = stl.vertices[triangle.vertices[0]];
        let v2 = stl.vertices[triangle.vertices[1]];
        let v3 = stl.vertices[triangle.vertices[2]];
        let points: Vec<Point3<f32>> = vec![v1, v2, v3].iter().map(|v| {
            Point3::new(v[0], v[1], v[2])
        }).collect();

        points
    }).collect::<Faces>();

    let mesh = Mesh::<String>::new(faces);

    let view = View::new(camera, Scene::new(vec![mesh]));
    let engine = Engine::new(vec![view]);
    engine.render("moon.svg".to_string())
}
