use super::{least_movement_path, movement_cost};
use crate::pitch_description::GuitarVoicing;

fn voicing(frets: [i8; 6], base_fret: u8) -> GuitarVoicing {
    GuitarVoicing {
        frets,
        base_fret,
        barres: &[],
    }
}

/// chords-db's open C (`x32010`) and its barred A-shape at fret 3.
fn c_open() -> GuitarVoicing {
    voicing([-1, 3, 2, 0, 1, 0], 1)
}

fn c_barre_8() -> GuitarVoicing {
    voicing([1, 3, 3, 2, 1, 1], 8)
}

/// An F barre at fret 8 (`x 8 10 10 10 8`), close to `c_barre_8`.
fn f_barre_8() -> GuitarVoicing {
    voicing([-1, 1, 3, 3, 3, 1], 8)
}

/// An F barre at fret 1, close to `c_open`.
fn f_barre_1() -> GuitarVoicing {
    voicing([1, 3, 3, 2, 1, 1], 1)
}

#[test]
fn the_same_voicing_costs_nothing() {
    assert_eq!(movement_cost(&c_open(), &c_open()), 0);
}

#[test]
fn moving_up_the_neck_costs_more_than_staying_near() {
    assert!(movement_cost(&c_open(), &f_barre_1()) < movement_cost(&c_open(), &f_barre_8()));
}

#[test]
fn picks_the_global_optimum_over_each_chords_first_voicing() {
    // Each chord's first voicing gives C(open) → F(8) → C(open): two big
    // jumps. Staying at fret 8 throughout (C(8) → F(8) → C(8)) moves less.
    let (c_open, c_barre_8, f_barre_8) = (c_open(), c_barre_8(), f_barre_8());
    let c = vec![&c_open, &c_barre_8];
    let f = vec![&f_barre_8];
    assert_eq!(
        least_movement_path(&[c.clone(), f, c.clone()]),
        vec![1, 0, 1]
    );
}

#[test]
fn a_repeated_chord_keeps_its_voicing() {
    let (c_open, c_barre_8, f_barre_1, f_barre_8) =
        (c_open(), c_barre_8(), f_barre_1(), f_barre_8());
    let c = vec![&c_open, &c_barre_8];
    let f = vec![&f_barre_8, &f_barre_1];
    assert_eq!(
        least_movement_path(&[c.clone(), c.clone(), f, c.clone(), c]),
        vec![0, 0, 1, 0, 0]
    );
}

#[test]
fn ties_resolve_to_chords_db_order() {
    let (c_open, c_barre_8) = (c_open(), c_barre_8());
    assert_eq!(least_movement_path(&[vec![&c_open, &c_barre_8]]), vec![0]);
    assert_eq!(least_movement_path(&[vec![&c_barre_8, &c_open]]), vec![0]);
    let same_twice = vec![&c_open, &c_open];
    assert_eq!(
        least_movement_path(&[same_twice.clone(), same_twice]),
        vec![0, 0]
    );
}

#[test]
fn no_chords_give_an_empty_path() {
    assert_eq!(least_movement_path(&[]), Vec::<usize>::new());
}
