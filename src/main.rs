use plotters::{prelude::*, style::full_palette::{GREY, ORANGE, BLUE_300, BLUE_100, PURPLE}};
mod orbitor;

use crate::orbitor::orbitor::{SolarSystemObject, SolarSystem, Orbitor};

fn main() {
    let mut solar_system = SolarSystem::new(8196, 20.0);
    let sun = SolarSystemObject::new_static(
        String::from("Sun"), 
        &YELLOW,
        1.9885e30,
        0.0,
        0.0,
        0.0);
    solar_system.add(&sun);
    let mercury = SolarSystemObject::new_orbitor(
        String::from("Mercury"),
        &WHITE,
        3.3011e23,
        &sun,
        5.7909e+7,
        0.2056,
        7.004,
        48.331,
        29.124,
        174.796,
    );
    solar_system.add(&mercury);
    let venus = SolarSystemObject::new_orbitor(
        String::from("Venus"),
        &PURPLE,
        4.8675e24,
        &sun,
        108208000.0,
        0.06772,
        3.39458,
        76.68,
        54.884,
        50.115,
    );
    solar_system.add(&venus);
    let earth = SolarSystemObject::new_orbitor(
        String::from("Earth"),
        &BLUE_300,
        5.97217e24,
        &sun,
        149598023.0,
        0.0167086,
        0.00005,
        -11.26064, 
        114.20783,
        358.617,
    );
    solar_system.add(&earth);
    let moon_loc = |time| Orbitor::new(
            7.342e22,
            &earth,
            384399.0,
            0.0549,
            5.145,
            125.08 - time * 360.0 / (18.61 * 1000.0),
            318.15 + time * 360.0 / (8.85 * 1000.0),
            135.27,
        );
    let moon = SolarSystemObject::new_variable(
        String::from("Moon"),
        &GREY,
        &moon_loc 
    );
    solar_system.add(&moon);
    let mars = SolarSystemObject::new_orbitor(
        String::from("Mars"),
        &RED,
        6.4171e23,
        &sun,
        227939366.0,
        0.0934,
        1.850,
        49.57854,
        286.5,
        19.412,
    );
    solar_system.add(&mars);
    let jupiter = SolarSystemObject::new_orbitor(
        String::from("Jupiter"),
        &ORANGE,
        1.8982e27,
        &sun,
        778479000.0,
        0.0489,
        1.303,
        100.464,
        273.867,
        20.020,
    );
    let saturn = SolarSystemObject::new_orbitor(
        String::from("Saturn"),
        &RGBColor(100, 100, 0),
        5.6834e24,
        &sun,
        1433530000.0,
        0.0565,
        2.485,
        113.665,
        339.392,
        317.020,
    );
    let uranus = SolarSystemObject::new_orbitor(
        String::from("Uranus"),
        &BLUE_100,
        8.6810e25,
        &sun,
        2870972000.0,
        0.04717,
        0.773,
        74.006, 
        96.998857,
        142.2386,
    );
    let neptune = SolarSystemObject::new_orbitor(
        String::from("Neptune"), 
        &BLUE,
        1.02413e26,
        &sun,
        4500000000.0,
        0.008678,
        1.770,
        131.783,
        273.187,
        256.228,
    );
    if solar_system.scale > 150.0 {
        solar_system.add(&jupiter);
        solar_system.add(&saturn);
        solar_system.add(&uranus);
        solar_system.add(&neptune);
    }
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
    let dim = 2;
    let rel = true;
    if rel {
        if dim == 2 {
            solar_system.plot_rel_2d(&earth, 0.0);
        }
        else {
            solar_system.plot_rel_3d(&earth, 0.0);
        }
    }
    else {
        if dim == 2 {
            solar_system.plot_2d(&earth, 0.0);
        }
        else {
            solar_system.plot_3d(&earth, 0.0);
        }
    }
    println!("Done");
}
