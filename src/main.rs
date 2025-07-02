use bevy::ecs::system::lifetimeless::SResMut;
use bevy::prelude::*;
use bevy::math::*;
use bevy::render::view::RenderLayers;
use bevy::time::common_conditions::on_timer;
use std::cmp;
use std::collections::*;
use std::num;
use std::time::Duration;
use rand::prelude::*;

const SIZE_X: i32 = 25;
const SIZE_Y: i32 = 25;
const SCALE: f32 = 15.;
const TEMP_DECAY: f32 = 0.01;
const TEMP_DIFFUSION: f32 = 0.1;
const BUG_HEAT: f32 = 2.5;
const BUG_MIN: f32 = 10.;
const BUG_MAX: f32 = 15.;
const BUG_NUMBER: i32 = 30;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Field(HashMap::new()))
        .add_systems(Startup, setup)
        .add_systems(
	    Update,
	    (temp_update, move_bugs, redraw).chain().run_if(on_timer(Duration::from_millis(100))),
	)
        .run();
}

#[derive(Component)]
struct Bug;

#[derive(Component)]
struct Cell;

#[derive(Component, Hash, PartialEq, Eq, Clone)]
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
    commands.spawn((
	Camera2dBundle {
	    camera: Camera {
		order: 1,
		..default()
	    },
	    ..default()
	},
	RenderLayers::layer(0),
    ));
    commands.spawn((
	Camera2dBundle {
	    camera: Camera {
		order: 2,
		..default()
	    },
	    ..default()
	},
	RenderLayers::layer(1),
    ));
    let mut rng = rand::rng();
    for i in 0..SIZE_X {
        for j in 0..SIZE_Y {
            let c = meshes.add(Rectangle::new(SCALE, SCALE));
            field.0.insert(
                IntCoords{x: i, y: j},
                commands.spawn((
                    Cell,
                    IntCoords{x: i, y: j},
                    Temperature(rng.random_range(0.0..5.0)),
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
	let c = meshes.add(Rectangle::new(SCALE, SCALE));
	let xr = rng.random_range(0..SIZE_X);
	let yr = rng.random_range(0..SIZE_Y);
	commands.spawn((
	    Bug,
	    IntCoords{x: xr, y: yr},
	    Mesh2d(c),
	    MeshMaterial2d(materials.add(Color::linear_rgb(0.,0.7,1.))),
	    Transform::from_xyz(
		SCALE * xr as f32 - SIZE_X as f32 * SCALE / 2.,
                SCALE * yr as f32 - SIZE_Y as f32 * SCALE / 2.,
                0.,
	    ),
	    RenderLayers::layer(1),
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
	    t.0 *= 1. - TEMP_DECAY - 8. * TEMP_DIFFUSION;
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
    for c in bugs.into_iter(){
	let mut t = cells.get_mut(*field.0.get(&IntCoords{x: c.x, y: c.y}).unwrap()).unwrap();
	t.0 += BUG_HEAT;
    }
}

fn move_bugs (
    field: Res<Field>,
    mut bugs: Query<&mut IntCoords, With<Bug>>,
    cells: Query<&Temperature, With<Cell>>,
) {
    for mut bug in &mut bugs {
	let mut t = cells.get(*field.0.get(&bug).unwrap()).unwrap();
	if t.0 > BUG_MAX || t.0 < BUG_MIN {
	    let mut best_move: IntCoords = bug.clone();
	    for x in -1..=1 {
		for y in -1..=1 {
		    let c = IntCoords {
			x: (x + bug.x + SIZE_X) % SIZE_X,
			y: (y + bug.y + SIZE_Y) % SIZE_Y,
		    };
		    let tn = cells.get(*field.0.get(&c).unwrap()).unwrap();
		    if tn.0 >= BUG_MIN && tn.0 <= BUG_MAX {
			if t.0 >= BUG_MIN && t.0 <= BUG_MAX {
			    if
				(tn.0 - BUG_MIN).abs().min((tn.0 - BUG_MAX).abs()) >
				(t.0 - BUG_MIN).abs().min((t.0 - BUG_MAX).abs())
			    {
				best_move = c.clone();
				t = tn;
			    }
			} else {
			    best_move = c.clone();
			    t = tn;
			}
		    } else if tn.0 < BUG_MIN || tn.0 > BUG_MAX {
			if t.0 < BUG_MIN || t.0 > BUG_MAX {
			    if
				(tn.0 - BUG_MIN).abs().min((tn.0 - BUG_MAX).abs()) <
				(t.0 - BUG_MIN).abs().min((t.0 - BUG_MAX).abs())
			    {
				best_move = c.clone();
				t = tn;
			    }
			}
		    }
		}
	    }
	    bug.x = best_move.x;
	    bug.y = best_move.y;
	}
    }
}

fn redraw (
    query: Query<(&mut MeshMaterial2d<ColorMaterial>, &Temperature), With<Cell>>,
    mut bugs: Query<(&mut Transform, &IntCoords), With<Bug>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for cell in &query {
        let material = materials.get_mut(cell.0.0.id()).unwrap();
        material.color = Color::linear_rgb(
	    1. - 1. / ((cell.1.0 + 1.) / 10.),
	    1. - 1. / ((cell.1.0 + 1.) / 10.),
	    1. - 1. / ((cell.1.0 + 1.) / 10.),
	);
    }
    for mut bug in &mut bugs {
	bug.0.translation = Vec3{
	    x : SCALE * bug.1.x as f32 - SIZE_X as f32 * SCALE / 2.,
            y : SCALE * bug.1.y as f32 - SIZE_Y as f32 * SCALE / 2.,
            z : 0.,
	};
    }
}
