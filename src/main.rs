//! Shows how to render to a texture. Useful for mirrors, UI, or exporting images.

mod post_process;

use bevy::input::common_conditions::input_toggle_active;
use bevy::{ window::WindowResized,
    core_pipeline::clear_color::ClearColorConfig,
    input::common_conditions::input_just_pressed,
    prelude::*,
    render::{
        camera::RenderTarget,
        extract_component::{ExtractComponentPlugin, ExtractComponent},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use post_process::{PostProcessPlugin};

fn main() {
    App::new()
        .register_type::<Dimensions>()
        .add_plugins(DefaultPlugins)
        .add_plugin(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_plugin(ExtractComponentPlugin::<Dimensions>::default())
        .add_plugin(PostProcessPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (rotator_system, move_system))
        .add_systems(
            Update,
            switch_dimension.run_if(input_just_pressed(KeyCode::D)),
        )
        .add_systems(Update, on_resize_system)
        .run();
}

#[derive(Component, Default, Debug, Clone, ExtractComponent, Reflect, FromReflect)]
struct Dimensions {
    dimensions: Vec<DimensionDef>,
    selected: u32,
}
#[derive(Default, Debug, Clone, Reflect, FromReflect)]
struct DimensionDef {
    image: Handle<Image>,
}

#[derive(Component, Reflect, FromReflect)]
struct Rotate(f32);
#[derive(Component, Reflect, FromReflect)]
struct Move;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 1280,
        height: 720,
        ..default()
    };
    let dimension_1_layer = RenderLayers::layer(1);
    let dimension_2_layer = RenderLayers::layer(2);
    let (image_handle_dimension_2, camera_2) = create_camera(
        size,
        &mut images,
        &mut meshes,
        &mut materials,
        &mut commands,
        dimension_2_layer,
    );
    let (image_handle_dimension_1, camera_1) = create_camera(
        size,
        &mut images,
        &mut meshes,
        &mut materials,
        &mut commands,
        dimension_1_layer,
    );
    // The main pass camera.
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Dimensions {
            dimensions: vec![
                DimensionDef {
                    image: image_handle_dimension_1,
                },
                DimensionDef {
                    image: image_handle_dimension_2,
                },
            ],
            selected: 0,
        }, 
        Move
    )).add_child(camera_1).add_child(camera_2);


    let quad_size = Vec2::new(250f32, 250f32);
    // The quad within dimension 1
    let mesh = meshes.add(shape::Quad::new(quad_size).into());
    let cube_material_handle = materials.add(ColorMaterial {
        color: Color::RED,
        texture: None,
    });
    commands.spawn((
        ColorMesh2dBundle {
            mesh: mesh.into(),
            material: cube_material_handle,
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
        },
        Rotate(1.8f32),
        dimension_1_layer,
    ));


    // The quad within dimension 2
    let mesh = meshes.add(shape::Quad::new(quad_size).into());
    let cube_material_handle = materials.add(ColorMaterial {
        color: Color::BLUE,
        texture: None,
    });
    commands.spawn((
        ColorMesh2dBundle {
            mesh: mesh.into(),
            material: cube_material_handle,
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
        },
        Rotate(1.5f32),
        dimension_2_layer,
    ));
}

fn create_camera(
    size: Extent3d,
    mut images: &mut Assets<Image>,
    mut meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    commands: &mut Commands<'_, '_>,
    render_layers: RenderLayers,
) -> (Handle<Image>, Entity) {
    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    let cam = commands
        .spawn((
            Camera2dBundle {
                camera_2d: Camera2d {
                    clear_color: ClearColorConfig::Custom(Color::BLACK),
                    ..default()
                },
                camera: Camera {
                    // render before the "main pass" camera
                    order: -1,
                    target: RenderTarget::Image(image_handle.clone()),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            render_layers,
        ))
        .id();

    (image_handle, cam)
}

/// Rotates the inner cube (first dimension)
fn rotator_system(time: Res<Time>, mut query: Query<(&mut Transform, &Rotate)>) {
    for (mut transform, rotate) in &mut query {
        //transform.rotate_x(1.5 * time.delta_seconds());
        transform.rotate_z(rotate.0 * time.delta_seconds());
    }
}
fn move_system(time: Res<Time>, mut query: Query<(&mut Transform, &Move)>) {
    for (mut transform, rotate) in &mut query {
        //transform.rotate_x(1.5 * time.delta_seconds());
        transform.translation.x = f32::sin(time.elapsed_seconds() * 5f32) * 300f32;
    }
}

fn switch_dimension(mut dim: Query<&mut Dimensions>) {
    for mut dimensions in dim.iter_mut() {
        let nb_dimensions = dimensions.dimensions.len() as u32;
        if nb_dimensions == 0 {
            return;
        }
        dimensions.selected = (dimensions.selected + 1) % nb_dimensions;
    }
}

/// This system shows how to respond to a window being resized.
/// Whenever the window is resized, the text will update with the new resolution.
fn on_resize_system(
    mut images: ResMut<Assets<Image>>,
    mut dim: Query<&Dimensions>,
    mut resize_reader: EventReader<WindowResized>,
) {
    if let Some(size) = resize_reader.iter().last() {
        let size = Extent3d {
            width: size.width as u32,
            height: size.height as u32,
            ..default()
        };
        for d in dim.iter() {
            for dimension in d.dimensions.iter() {
                if let Some(mut image) = images.get_mut(&dimension.image) {
                
                    image.resize(size);
                }
            }
        }
    }
}