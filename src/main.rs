use bevy::prelude::*;
use bevy::math::*;
use std::collections::*;
use rand::prelude::*;

const SIZE_X: i32 = 20;
const SIZE_Y: i32 = 20;
const SCALE: f32 = 10.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Field(HashMap::new()))
        .add_systems(Startup, setup)
        .add_systems(Update, redraw)
        .run();
}

#[derive(Component)]
struct Cell;

#[derive(Component, Hash, PartialEq, Eq)]
struct IntCoords {x: i32, y: i32}

#[derive(Component)]
struct Temperature (f32);

#[derive(Resource)]
struct Field (HashMap<IntCoords, Entity>);

fn setup (
    mut field: ResMut<Field>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    let mut rng = rand::rng();
    for i in 0..SIZE_X {
        for j in 0..SIZE_Y {
            let c = meshes.add(Rectangle::new(SCALE, SCALE));
            field.0.insert(
                IntCoords{x: i, y: j},
                commands.spawn((
                    Cell,
                    IntCoords{x: i, y: j}, 
                    Temperature(rng.random_range(0.0..100.0)),
                    Mesh2d(c),
                    MeshMaterial2d(materials.add(Color::linear_rgb(0.,0.,0.))),
                    Transform::from_xyz(
                        SCALE * i as f32 - SIZE_X as f32 * SCALE / 2.,
                        SCALE * j as f32 - SIZE_Y as f32 * SCALE / 2.,
                        0.,
                    ),
                )).id()
            );
        }
    }
}

fn redraw (
    mut query: Query<(&mut MeshMaterial2d<ColorMaterial>, &Temperature), With<Cell>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for mut cell in &query {
        let material = materials.get_mut(cell.0.0.id()).unwrap();
        material.color = Color::linear_rgb(1. - 1. / ((cell.1.0 + 1.) / 10.), 0., 0.);
    }
}