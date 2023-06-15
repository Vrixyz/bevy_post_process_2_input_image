//! Shows how to render to a texture. Useful for mirrors, UI, or exporting images.

mod post_process;

use std::f32::consts::PI;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        camera::RenderTarget,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};
use post_process::{PostProcessPlugin, PostProcessSettings};

fn main() {
    App::new()
        .init_resource::<Dimension1>()
        .init_resource::<Dimension2>()
        .add_plugins(DefaultPlugins)
        .add_plugin(ExtractResourcePlugin::<Dimension1>::default())
        .add_plugin(ExtractResourcePlugin::<Dimension2>::default())
        .add_plugin(PostProcessPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (rotator_system, show_dim))
        .run();
}

#[derive(Resource, Default, Debug, Clone, ExtractResource)]
struct Dimension1 {
    image: Option<Handle<Image>>,
}
#[derive(Resource, Default, Clone, ExtractResource)]
struct Dimension2 {
    image: Option<Handle<Image>>,
}

// Marks the first pass cube (rendered to a texture.)
#[derive(Component)]
struct FirstPassCube;

// Marks the main pass cube, to which the texture is applied.
#[derive(Component)]
struct MainPassCube;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let quad_size = 150f32;
    let size = Extent3d {
        width: quad_size as u32,
        height: quad_size as u32,
        ..default()
    };
    let dimension_1_layer = RenderLayers::layer(1);

    // The quad within dimension 1
    let mesh =
        meshes.add(shape::Quad::new(Vec2::new(size.width as f32, size.height as f32)).into());
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
        FirstPassCube,
        dimension_1_layer,
    ));
    let image_handle_dimension_1 = create_camera(
        size,
        &mut images,
        &mut meshes,
        &mut materials,
        &mut commands,
        dimension_1_layer,
    );
    // This material has the texture that has been rendered for dimension 1.
    let material_handle_dim1 = materials.add(ColorMaterial {
        color: Color::default(),
        texture: Some(image_handle_dimension_1.clone()),
    });

    let dimension_2_layer = RenderLayers::layer(2);

    // The quad within dimension 1
    let mesh =
        meshes.add(shape::Quad::new(Vec2::new(size.width as f32, size.height as f32)).into());
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
        FirstPassCube,
        dimension_2_layer,
    ));
    let image_handle_dimension_2 = create_camera(
        size,
        &mut images,
        &mut meshes,
        &mut materials,
        &mut commands,
        dimension_2_layer,
    );

    // This material has the texture that has been rendered for dimension 1.
    let material_handle_dim2 = materials.add(ColorMaterial {
        color: Color::default(),
        texture: Some(image_handle_dimension_2.clone()),
    });

    let quad_handle = meshes.add(shape::Quad::new(Vec2::new(quad_size, quad_size)).into());
    // Main pass quad, with material containing the rendered first dimension texture.
    commands.spawn((
        ColorMesh2dBundle {
            mesh: quad_handle.clone().into(),
            material: material_handle_dim1,
            transform: Transform::from_xyz(-150.0, 0.0, 1.5),
            ..default()
        },
        MainPassCube,
    ));
    commands.spawn((
        ColorMesh2dBundle {
            mesh: quad_handle.into(),
            material: material_handle_dim2,
            transform: Transform::from_xyz(150.0, 0.0, 1.5),
            ..default()
        },
        MainPassCube,
    ));

    // The main pass camera.
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PostProcessSettings { intensity: 0.02 },
    ));

    commands.insert_resource(Dimension1 {
        image: Some(image_handle_dimension_1),
    });
    commands.insert_resource(Dimension2 {
        image: Some(image_handle_dimension_2),
    });
}

fn create_camera(
    size: Extent3d,
    mut images: &mut Assets<Image>,
    mut meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    commands: &mut Commands<'_, '_>,
    render_layers: RenderLayers,
) -> Handle<Image> {
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

    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::WHITE),
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
    ));

    image_handle
}

/// Rotates the inner cube (first dimension)
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<FirstPassCube>>) {
    for mut transform in &mut query {
        //transform.rotate_x(1.5 * time.delta_seconds());
        transform.rotate_z(1.3 * time.delta_seconds());
    }
}

fn show_dim(dim: Res<Dimension1>) {
    //dbg!(dim);
}
