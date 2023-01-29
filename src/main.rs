use plotters::{prelude::*, style::full_palette::{GREY, ORANGE, BLUE_300}};
mod orbitor;

use crate::orbitor::orbitor::{SolarSystemObject, SolarSystem, Locatable};
fn main() {
    let mut solar_system = SolarSystem::new();
    let sun = SolarSystemObject::new_static(
        "Sun", 
        &YELLOW,
        1.9885e30,
        0.0, 
        0.0, 
        0.0);
    solar_system.add(&sun);
    let mercury = SolarSystemObject::new_orbitor(
        "Mercury",
        &WHITE,
        3.3011e23,
        &sun,
        5.7909e+7,
        0.2056,
        7.004,
        48.331,
        29.124,
        0.0,
    );
    solar_system.add(&mercury);
    let venus = SolarSystemObject::new_orbitor(
        "Venus",
        &GREEN,
        4.8675e24,
        &sun,
        108208000.0,
        0.06772,
        3.39458,
        76.68,
        54.884,
        0.0,
    );
    solar_system.add(&venus);
    let earth = SolarSystemObject::new_orbitor(
        "Earth",
        &BLUE_300,
        5.97217e24,
        &sun,
        149598023.0,
        0.0167086,
        0.00005,
        -11.26064, 
        114.20783,
        0.0,
    );
    solar_system.add(&earth);
    let moon = SolarSystemObject::new_orbitor(
        "Moon",
        &GREY,
        7.342e22,
        &earth,
        384399.0,
        0.0549,
        5.145,
        0.0, // TODO make these able to be functions of time?
        0.0,
        0.0,
    );
    solar_system.add(&moon);
    let mars = SolarSystemObject::new_orbitor(
        "Mars",
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
        "Jupiter",
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
    solar_system.add(&jupiter);
    let saturn = SolarSystemObject::new_orbitor(
        "Saturn",
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
    solar_system.add(&saturn);
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
    let root_drawing_area = BitMapBackend::new("images/0.1.png", (4096, 4096))
        .into_drawing_area();

    root_drawing_area.fill(&BLACK).unwrap();
    let chart_size: f64 = 200.0;
    let mut chart = ChartBuilder::on(&root_drawing_area)
        .build_cartesian_2d(-chart_size..chart_size, -chart_size..chart_size)
        .unwrap();

    for obj in solar_system.objects() {
        chart.draw_series(LineSeries::new(
            obj.orbit_points(20000),
            // (-5000..5000).map(|x| x as f64 / 10.0).map(|x| obj.xy(x)),
            obj.get_color(),
        )).unwrap();
    }
}
