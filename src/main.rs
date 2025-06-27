use bevy::ecs::system::lifetimeless::SResMut;
use bevy::prelude::*;
use bevy::math::*;
use std::collections::*;
use rand::prelude::*;

const SIZE_X: i32 = 20;
const SIZE_Y: i32 = 20;
const SCALE: f32 = 10.;
const TEMP_DECAY: f32 = 0.01;
const TEMP_DIFFUSION: f32 = 0.1;
const BUG_HEAT: f32 = 15.;
const BUG_MIN: f32 = 10.;
const BUG_MAX: f32 = 40.;
const BUG_NUMBER: i32 = 30;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Field(HashMap::new()))
        .add_systems(Startup, setup)
        .add_systems(Update, (redraw, temp_update).chain())
        .run();
}

#[derive(Component)]
struct Bug;

#[derive(Component)]
struct Cell;

#[derive(Component, Hash, PartialEq, Eq)]
struct IntCoords {x: i32, y: i32}

#[derive(Component, Clone)]
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
                    Temperature(rng.random_range(0.0..50.0)),
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
    for _ in 0..BUG_NUMBER {
	commands.spawn((
	    Bug,
	    IntCoords{x: rng.random_range(0..SIZE_X), y: rng.random_range(0..SIZE_Y)},
	));
    }
}

fn temp_update (
    field: Res<Field>,
    bugs: Query<&IntCoords, With<Bug>>,
    mut cells: Query<&mut Temperature, With<Cell>>,
) {
    let mut field_tmp: HashMap<IntCoords, Temperature> = Default::default();
    for i in 0..SIZE_X {
	for j in 0..SIZE_Y {
	    field_tmp.insert(
		IntCoords{x: i, y: j},
		cells.get(*field.0.get(&IntCoords{x: i, y: j}).unwrap()).unwrap().clone(),
	    );
	}
    }
    for i in 0..SIZE_X {
	for j in 0..SIZE_Y {
	    let mut t = cells.get_mut(*field.0.get(&IntCoords{x: i, y: j}).unwrap()).unwrap();
	    t.0 *= (1. - TEMP_DECAY - 8. * TEMP_DIFFUSION);
	    for x in -1..=1 {
		for y in -1..=1 {
		    if let Some(n) = field_tmp.get(&IntCoords{
			x: (x + i + SIZE_X) % SIZE_X,
			y: (y + j + SIZE_Y) % SIZE_Y,
		    }) {
			if !(x == 0 && y == 0){
			    t.0 += n.0 * TEMP_DIFFUSION;
			}
		    } else {
			panic!("neighbour not found");
		    }
		    
		}
	    }
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
