use plotters::{prelude::*, style::full_palette::GREY};
mod orbitor;
use std::f64::consts::TAU;

use crate::orbitor::orbitor::{StaticObject, Orbitor, SolarSystemObject, Locatable};

fn deg_to_rad(x: f64) -> f64 {
    x * TAU / 360.0
}

fn main() {
    let sun = StaticObject::new(0.0, 0.0, 0.0);
    let sun_wrap = SolarSystemObject::Static { s: &sun };
    let mercury = Orbitor::new(
        &sun_wrap,
        5.7909e+7,
        0.2056,
        deg_to_rad(7.004),
        deg_to_rad(48.331),
        deg_to_rad(29.124),
        0.0,
    );
    let venus = Orbitor::new(
        &sun_wrap,
        108208000.0,
        0.06772,
        deg_to_rad(3.39458),
        deg_to_rad(76.68),
        54.884,
        0.0,
    );
    let earth = Orbitor::new(
        &sun_wrap,
        149598023.0,
        0.0167086,
        deg_to_rad(0.00005),
        deg_to_rad(-11.26064), 
        deg_to_rad(114.20783),
        0.0,
    );
    let earth_wrap = SolarSystemObject::Moving { o: &earth };
    let moon = Orbitor::new(
        &earth_wrap,
        3843990.0,
        0.0549,
        deg_to_rad(5.145),
        0.0, // TODO make these able to be functions of time?
        0.0,
        0.0
    );
    let (x, y, z) = sun.xyz(10.0);
    let (x2, y2, z2) = mercury.xyz(10.0);
    println!("{:?}", (-1000..1000)
        .map(|x| x as f64 / 5.0)
        .map(|x| mercury.xy(x))
        .map(|(x, y)| (x*x+y*y).sqrt())
        .max_by(|x, y| x.partial_cmp(y).unwrap()).unwrap());
    println!("{:?}", (-1000..1000)
        .map(|x| x as f64 / 5.0)
        .map(|x| mercury.xy(x))
        .map(|(x, y)| (x*x+y*y).sqrt())
        .min_by(|x, y| x.partial_cmp(y).unwrap()).unwrap());
    println!("{} {} {}; {} {} {}", x, y, z, x2, y2, z2);
    let root_drawing_area = BitMapBackend::new("images/0.1.png", (2048, 2048))
        .into_drawing_area();

    root_drawing_area.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root_drawing_area)
        .build_cartesian_2d(-200.0..200.0 as f64, -200.0..200.0 as f64)
        .unwrap();
 
    chart.draw_series(LineSeries::new(
        (0..100).map(|x| x as f64).map(|x| sun.xy(x)),
        &BLACK
    )).unwrap();

    chart.draw_series(LineSeries::new(
        (-1000..1000).map(|x| x as f64 / 5.0).map(|x| mercury.xy(x)),
        &RED
    )).unwrap();

    chart.draw_series(LineSeries::new(
        (-1000..1000).map(|x| x as f64).map(|x| venus.xy(x)),
        &MAGENTA
    )).unwrap();

    chart.draw_series(LineSeries::new(
        (-10000..10000).map(|x| x as f64 / 20.0).map(|x| moon.xy(x)),
        &GREY
    )).unwrap();  

    chart.draw_series(LineSeries::new(
        (-1000..1000).map(|x| x as f64).map(|x| earth.xy(x)),
        &BLUE
    )).unwrap();
 
}
