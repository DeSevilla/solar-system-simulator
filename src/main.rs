use std::{collections::HashMap, vec};

use plotters::{prelude::*, style::full_palette::{GREY, ORANGE, BLUE_300, BLUE_100, PURPLE}};
mod orbitor;

use crate::orbitor::orbitor::{SolarSystemObject, SolarSystem, Locatable, deg_to_rad};

const TIME: f64 = 100.0;
const PIXELS: u32 = 8192;
const DIMENSIONS: u32 = 2;
const STROKE_WIDTH_BASE: u32 = PIXELS / 2048; 

fn plot_2d(solar_system: SolarSystem, zodiac: HashMap<i32, String>, earth: &SolarSystemObject) {
    
    println!("Drawing...");

    let (ex, ey) = earth.xy(TIME);

    // let root_drawing_area = SVGBackend::new("images/solar_system.svg", (PIXELS, PIXELS))
    //     .into_drawing_area();
    let root_drawing_area = BitMapBackend::new("images/solar_system.png", (PIXELS, PIXELS))
        .into_drawing_area();

    root_drawing_area.fill(&BLACK).unwrap();
    let chart_size: f64 = 200.0;
    let mut chart = ChartBuilder::on(&root_drawing_area)
        .build_cartesian_2d(-chart_size..chart_size, -chart_size..chart_size)
        .unwrap();

    for angle in zodiac.keys() {
        let angle_rad = deg_to_rad(*angle as f64);
        let far_edge = (ex + chart_size * angle_rad.cos(), ey + chart_size * angle_rad.sin());
        chart.draw_series(LineSeries::new(
            vec![(ex, ey), far_edge],
            Into::<ShapeStyle>::into(&GREY).stroke_width(STROKE_WIDTH_BASE),
        )).unwrap();
    }
    for obj in solar_system.objects() {
        let (ox, oy) = obj.xy(TIME);
        chart.draw_series(PointSeries::of_element(
            vec![(ox, oy)], 
            STROKE_WIDTH_BASE * 5, 
            Into::<ShapeStyle>::into(obj.get_color()).filled(),
            &|coord, size, style| {
                EmptyElement::at(coord)
                + Circle::new(
                    (0, 0), 
                    size, 
                    style
                )
            }
        )).unwrap();
        let angle = obj.angle_deg(earth, TIME);
        chart.draw_series(LineSeries::new(
            vec![(ex, ey), (ox, oy)],
            Into::<ShapeStyle>::into(obj.get_color()).stroke_width(STROKE_WIDTH_BASE)
        )).unwrap();
        // chart.draw_series(LineSeries::new(
        //     vec![(x2, y2), (x2 + 50.0 * angle_rad.cos(), y2 + 50.0 * angle_rad.sin())], 
        //     obj.get_color()
        // )).unwrap();
        // let angle = obj.angle_deg(0.0);
        let angle_rounded = (angle / 30.0).floor() as i32 * 30;
        let sign = zodiac.get(&angle_rounded);
        println!("{}: {} ({}, {:?})", obj.get_name(), angle, angle_rounded, sign);
        // for point in obj.trajectory_2d(10) {
        //     println!("{:?}", point);
        // }
        let stroke_width = if obj.get_name() == "Moon" {1} else {2};
        chart.draw_series(LineSeries::new(
            obj.trajectory_2d(200),
            // (-5000..5000).map(|x| x as f64 / 10.0).map(|x| obj.xy(x)),
            Into::<ShapeStyle>::into(obj.get_color()).stroke_width(STROKE_WIDTH_BASE * stroke_width),
        )).unwrap();
    }
}

fn plot_3d(solar_system: SolarSystem, zodiac: HashMap<i32, String>, earth: &SolarSystemObject) {
    
    let (ex, ey, ez) = earth.xyz(TIME);
    println!("Drawing...");

    let root_drawing_area = SVGBackend::new("images/solar_system_3d.svg", (PIXELS, PIXELS))
        .into_drawing_area();

    root_drawing_area.fill(&BLACK).unwrap();
    let chart_size: f64 = 200.0;
    let mut chart = ChartBuilder::on(&root_drawing_area).margin(20).caption("Solar system", ("sans-serif", 20))
        .build_cartesian_3d(-chart_size..chart_size, -50.0..50.0, -chart_size..chart_size)
        .unwrap();
    chart.with_projection(|mut pb| {
        pb.pitch = 0.2;
        pb.yaw = 1.0;
        pb.scale = 1.3;
        pb.into_matrix()
    });
    
    chart.configure_axes().draw().unwrap();

    for obj in solar_system.objects() {
        let (ox, oy, oz) = obj.xyz(TIME);
        let angle = obj.angle_deg(earth, TIME);
        chart.draw_series(LineSeries::new(
            vec![(ex, ey, ez), (ox, oy, oz)],
            Into::<ShapeStyle>::into(obj.get_color()).stroke_width(STROKE_WIDTH_BASE)
        )).unwrap();
        let angle_rounded = (angle / 30.0).floor() as i32 * 30;
        let sign = zodiac.get(&angle_rounded);
        println!("{}: {} ({}, {:?})", obj.get_name(), angle, angle_rounded, sign);
        // for point in obj.trajectory_3d(5) {
        //     println!("{:?}", point);
        // }
        // for point in (0..10).map(|i| i as f64 * TAU / 10.0).map(|i| match obj.xy(0.0) { (x, y) => (x + 5.0 * i.cos(), y + 5.0 * i.sin())}) {
        //     print!("{},{} ", point.0, point.1);
        // }
        let stroke_width = if obj.get_name() == "Moon" {1} else {2};
        chart.draw_series(LineSeries::new(
            // (0..11).map(|i| i as f64 * TAU / 10.0).map(|i| match obj.xy(TIME) { (x, y) => (x + 10.0 * i.cos(), y + 10.0 * i.sin())}),
            obj.trajectory_3d(45),
            // (-5000..5000).map(|x| x as f64 / 10.0).map(|x| obj.xy(x)),
            Into::<ShapeStyle>::into(obj.get_color()).stroke_width(STROKE_WIDTH_BASE * stroke_width),
        )).unwrap();
    }
}

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
        174.796,
    );
    solar_system.add(&mercury);
    let venus = SolarSystemObject::new_orbitor(
        "Venus",
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
        "Earth",
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
    // solar_system.add(&moon);
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
    let uranus = SolarSystemObject::new_orbitor(
        "Uranus",
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
    solar_system.add(&uranus);
    let neptune = SolarSystemObject::new_orbitor(
        "Neptune", 
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
    solar_system.add(&neptune);
    let mut zodiac = HashMap::new();
    zodiac.insert(0, String::from("Aries"));
    zodiac.insert(30, String::from("Taurus"));
    zodiac.insert(60, String::from("Gemini"));
    zodiac.insert(90, String::from("Cancer"));
    zodiac.insert(120, String::from("Leo"));
    zodiac.insert(150, String::from("Virgo"));
    zodiac.insert(180, String::from("Libra"));
    zodiac.insert(210, String::from("Scorpio"));
    zodiac.insert(240, String::from("Sagittarius"));
    zodiac.insert(270, String::from("Capricorn"));
    zodiac.insert(300, String::from("Aquarius"));
    zodiac.insert(330, String::from("Pisces"));
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
    if DIMENSIONS == 2 {
        plot_2d(solar_system, zodiac, &earth);
    }
    else if DIMENSIONS == 3 {
        plot_3d(solar_system, zodiac, &earth)
    }
    else {
        println!("We can't plot the solar system in {} dimensions, sorry", DIMENSIONS);
    }
    println!("Done");
}
