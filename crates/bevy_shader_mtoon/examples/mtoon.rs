use std::f32::consts::PI;

use bevy::{
    color::palettes::css::{BISQUE, SALMON},
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

use bevy_egui::{
    EguiContexts, EguiPlugin,
    egui::{Slider, Window},
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_shader_mtoon::{MtoonBundle, MtoonMaterial, MtoonPlugin, MtoonSun, VrmOutlineMode};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.1, 0.1, 0.1)))
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            EguiPlugin {
                enable_multipass_for_primary_context: false,
            },
            MtoonPlugin,
            PanOrbitCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate, ui))
        .run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mtoon_materials: ResMut<Assets<MtoonMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Transform::from_xyz(0.0, 8.0, 14.0),
        PanOrbitCamera {
            focus: Vec3::new(0.0, 1.0, 0.0),
            ..default()
        },
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-PI / 3.0)),
        MtoonSun,
    ));

    let mtoon_textured = MtoonBundle {
        mtoon: MeshMaterial3d(mtoon_materials.add(MtoonMaterial {
            base_color_texture: Some(images.add(uv_debug_texture())),
            outline_width: 0.002,
            outline_mode: VrmOutlineMode::Screen,
            ..default()
        })),
        ..default()
    };

    let mtoon_plain = MtoonBundle {
        mtoon: MeshMaterial3d(mtoon_materials.add(MtoonMaterial {
            base_color: BISQUE.into(),
            shade_factor: SALMON.into(),
            outline_width: 0.2,
            outline_mode: VrmOutlineMode::World,
            ..default()
        })),
        ..default()
    };

    let shapes = [
        meshes.add(Cuboid::default()),
        meshes.add(Capsule3d::default()),
        meshes.add(Torus::default()),
        meshes.add(Cylinder::default()),
        meshes.add(Sphere::default()),
    ];

    let num_shapes = shapes.len();

    // Spacing between shapes
    const X_EXTENT: f32 = 10.0;

    for (i, mesh) in shapes.into_iter().enumerate() {
        // Texture
        commands.spawn((
            Mesh3d(mesh.clone()),
            mtoon_textured.clone(),
            Transform::from_xyz(
                -X_EXTENT / 2.0 + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                1.0,
                3.0,
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.0)),
        ));

        // Without texture
        commands.spawn((
            Mesh3d(mesh),
            mtoon_plain.clone(),
            Transform::from_xyz(
                -X_EXTENT / 2.0 + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                1.0,
                -3.0,
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.0)),
        ));
    }

    // Big shape to test shadows.
    commands.spawn((
        Mesh3d(meshes.add(Torus::default())),
        mtoon_textured.clone(),
        Transform::from_xyz(0.0, 5.0, 0.0).with_scale(Vec3::splat(4.25)),
    ));

    // Ground
    commands.spawn((
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Mesh3d(meshes.add(Plane3d::default())),
        Transform::from_scale(Vec3::splat(30.0)),
    ));
}

fn rotate(time: Res<Time>, mut query: Query<&mut Transform, With<MeshMaterial3d<MtoonMaterial>>>) {
    for mut transform in query.iter_mut() {
        transform.rotate(Quat::from_rotation_y(time.delta_secs() / 2.0));
    }
}

fn ui(
    mut contexts: EguiContexts,
    mut mtoon_materials: ResMut<Assets<MtoonMaterial>>,
    mut settings: Local<MtoonMaterial>,
) {
    for (_, material) in mtoon_materials.iter_mut() {
        material.gi_equalization_factor = settings.gi_equalization_factor;
        material.parametric_rim_fresnel_power = settings.parametric_rim_fresnel_power;
        material.parametric_rim_lift_factor = settings.parametric_rim_lift_factor;
        material.rim_lighting_mix_factor = settings.rim_lighting_mix_factor;
        material.shading_shift_factor = settings.shading_shift_factor;
        material.shading_toony_factor = settings.shading_toony_factor;
    }

    Window::new("bevy_shader_mtoon").show(contexts.ctx_mut(), |ui| {
        ui.add(
            Slider::new(&mut settings.gi_equalization_factor, 0.0..=1.0)
                .text("GL Equalization Factor"),
        );

        ui.add(
            Slider::new(&mut settings.parametric_rim_fresnel_power, 0.0..=10.0)
                .text("Parametric Rim Fresnel Power"),
        );

        ui.add(
            Slider::new(&mut settings.parametric_rim_lift_factor, 0.0..=1.0)
                .text("Parametric Rim Lift Factor"),
        );

        ui.add(
            Slider::new(&mut settings.rim_lighting_mix_factor, 0.0..=1.0)
                .text("Rim Lighting Mix Factor"),
        );

        ui.add(
            Slider::new(&mut settings.shading_shift_factor, -1.0..=1.0)
                .text("Shading Shift Factor"),
        );

        ui.add(
            Slider::new(&mut settings.shading_toony_factor, 0.0..=1.0).text("Shading Toony Factor"),
        );
    });
}

fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
