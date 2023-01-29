use plotters::{prelude::*, style::full_palette::{GREY, ORANGE}};
mod orbitor;
use std::f64::consts::TAU;

use crate::orbitor::orbitor::{SolarSystemObject, SolarSystem, Locatable};

fn deg_to_rad(x: f64) -> f64 {
    x * TAU / 360.0
}

fn main() {
    let mut solar_system = SolarSystem::new();
    let sun = SolarSystemObject::new_static("Sun", &YELLOW, 0.0, 0.0, 0.0);
    solar_system.add(&sun);
    let mercury = SolarSystemObject::new_orbitor(
        "Mercury",
        &WHITE,
        &sun,
        5.7909e+7,
        0.2056,
        deg_to_rad(7.004),
        deg_to_rad(48.331),
        deg_to_rad(29.124),
        0.0,
    );
    solar_system.add(&mercury);
    let venus = SolarSystemObject::new_orbitor(
        "Venus",
        &GREEN,
        &sun,
        108208000.0,
        0.06772,
        deg_to_rad(3.39458),
        deg_to_rad(76.68),
        54.884,
        0.0,
    );
    solar_system.add(&venus);
    let earth = SolarSystemObject::new_orbitor(
        "Earth",
        &BLUE,
        &sun,
        149598023.0,
        0.0167086,
        deg_to_rad(0.00005),
        deg_to_rad(-11.26064), 
        deg_to_rad(114.20783),
        0.0,
    );
    solar_system.add(&earth);
    let moon = SolarSystemObject::new_orbitor(
        "Moon",
        &GREY,
        &earth,
        3843990.0,
        0.0549,
        deg_to_rad(5.145),
        0.0, // TODO make these able to be functions of time?
        0.0,
        0.0,
    );
    solar_system.add(&moon);
    let mars = SolarSystemObject::new_orbitor(
        "Mars",
        &RED, &sun,
        227939366.0,
        0.0934,
        deg_to_rad(1.850),
        deg_to_rad(49.57854),
        deg_to_rad(286.5),
        deg_to_rad(19.412),
    );
    solar_system.add(&mars);
    let jupiter = SolarSystemObject::new_orbitor(
        "Jupiter",
        &ORANGE,
        &sun,
        778479000.0,
        deg_to_rad(0.0489),
        deg_to_rad(1.303),
        deg_to_rad(100.464),
        deg_to_rad(273.867),
        deg_to_rad(20.020),
    );
    solar_system.add(&jupiter);
    // let (x, y, z) = sun.xyz(10.0);
    // let (x2, y2, z2) = mercury.xyz(10.0);
    // println!("{:?}", (-1000..1000)
    //     .map(|x| x as f64 / 5.0)
    //     .map(|x| mercury.xy(x))
    //     .map(|(x, y)| (x*x+y*y).sqrt())
    //     .max_by(|x, y| x.partial_cmp(y).unwrap()).unwrap());
    // println!("{:?}", (-1000..1000)
    //     .map(|x| x as f64 / 5.0)
    //     .map(|x| mercury.xy(x))
    //     .map(|(x, y)| (x*x+y*y).sqrt())
    //     .min_by(|x, y| x.partial_cmp(y).unwrap()).unwrap());
    // println!("{} {} {}; {} {} {}", x, y, z, x2, y2, z2);
    println!("Drawing...");
    let root_drawing_area = BitMapBackend::new("images/0.1.png", (2048, 2048))
        .into_drawing_area();

    root_drawing_area.fill(&BLACK).unwrap();

    let mut chart = ChartBuilder::on(&root_drawing_area)
        .build_cartesian_2d(-1000.0..1000.0 as f64, -1000.0..1000.0 as f64)
        .unwrap();

    for obj in solar_system.objects() {
        chart.draw_series(LineSeries::new(
            (-5000..5000).map(|x| x as f64 / 10.0).map(|x| obj.xy(x)),
            obj.get_color(),
        )).unwrap();
    }
}
