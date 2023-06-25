use std::env;
use std::time::SystemTime;
use time::{
    OffsetDateTime, 
    format_description::well_known::{Iso8601, Rfc3339, Rfc2822}, 
    Date, 
    macros::format_description
};
use plotters::{prelude::*,  style::full_palette::GREY};

mod orbitor;

use crate::orbitor::orbitor::{
    SolarSystem,
    Locatable,
    J2000, 
    Point2D, Point3D,
    deg_to_rad,
};

fn parse_time(time_str: &str) -> Result<OffsetDateTime, String> {
    if let Ok(time) = OffsetDateTime::parse(time_str, &Iso8601::DEFAULT) {
        Ok(time)
    }
    else if let Ok(time) = OffsetDateTime::parse(time_str, &Rfc2822) {
        Ok(time)
    }
    else if let Ok(time) = OffsetDateTime::parse(time_str, &Rfc3339) {
        Ok(time)
    }
    else if let Ok(date) = Date::parse(time_str, format_description!("[year]-[month]-[day]")) {
        Ok(date.midnight().assume_utc())
    }
    else if time_str == "now" {
        Ok(SystemTime::now().into())
    }
    else {
        Err("Unknown".into())
    }
}


pub fn plot_2d<'a>(solar_system: &'a SolarSystem<'a>, pixels: u32, scale: f64, time: f64) {
    let stroke_width_base = (pixels / 2048).max(1);
    
    println!("Drawing 2d absolute...");

    let Point2D(ex, ey) = solar_system.zodiac_center().xy(time);

    let root_drawing_area = SVGBackend::new("images/solar_system.svg", (pixels, pixels))
        .into_drawing_area();
    // let root_drawing_area = BitMapBackend::new("images/solar_system.png", (pixels, pixels))
    //     .into_drawing_area();

    root_drawing_area.fill(&BLACK).unwrap();
    let mut chart = ChartBuilder::on(&root_drawing_area)
        .build_cartesian_2d(-scale..scale, -scale..scale)
        .unwrap();

    for angle in solar_system.zodiac().angles() {
        let angle_rad = deg_to_rad(angle);
        let dx = angle_rad.cos();
        let dy = angle_rad.sin();
        let far_edge = (ex + scale * dx, ey + scale * dy);
        chart.draw_series(LineSeries::new(
            vec![(ex, ey), far_edge],
            Into::<ShapeStyle>::into(&GREY).stroke_width(stroke_width_base),
        )).unwrap();
    }
    for obj in solar_system.objects() {
        let Point2D(ox, oy) = obj.xy(time);
        chart.draw_series(PointSeries::of_element(
            vec![(ox, oy)],
            stroke_width_base * 5,
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
        chart.draw_series(LineSeries::new(
            vec![(ex, ey), (ox, oy)],
            Into::<ShapeStyle>::into(obj.get_color()).stroke_width(stroke_width_base)
        )).unwrap();
        let stroke_width = if obj.get_name() == "Moon" {1} else {2};
        let end_time = match obj.orbital_period(time) {
            Some(op) => time + op,
            None => time + 5.0
        };
        let trajectory: Vec<Point2D> = obj.trajectory(time, end_time, 100)
            .into_iter().map(|x| x.into()).collect();
        chart.draw_series(LineSeries::new(
            trajectory.into_iter().map(|x| x.loc()),
            Into::<ShapeStyle>::into(obj.get_color()).stroke_width(stroke_width_base * stroke_width),
        )).unwrap();
    }
}

pub fn plot_rel_2d<'a>(solar_system: &'a SolarSystem<'a>, pixels: u32, scale: f64, start_time: f64) {
    
    let stroke_width_base = (pixels / 2048).max(1);
    println!("Drawing 2d relative...");

    let root_drawing_area = SVGBackend::new("images/solar_system.svg", (pixels, pixels))
    // let root_drawing_area = BitMapBackend::new("images/solar_system.png", (pixels, pixels))
    // let root_drawing_area = BitMapBackend::gif("images/solar_system_anim.gif", (pixels, pixels), 100).unwrap()
        .into_drawing_area();

    let mut chart = ChartBuilder::on(&root_drawing_area)
        .build_cartesian_2d(-scale..scale, -scale..scale)
        .unwrap();

    for angle in solar_system.zodiac().angles() {
        let angle_rad = deg_to_rad(angle as f64);
        let dx = angle_rad.cos();
        let dy = angle_rad.sin();
        let far_edge = solar_system.zodiac_center().xy(start_time) + Point2D(scale * dx, scale * dy);
        chart.draw_series(LineSeries::new(
            vec![solar_system.zodiac_center().xy(start_time).loc(), far_edge.loc()],
            Into::<ShapeStyle>::into(&GREY).stroke_width(stroke_width_base),
        )).unwrap();
    }
    for i in 0..400 {
        if i % 10 == 0 {
            println!("{i}");
        }
        let time = start_time - (i * 5) as f64;
        let offset = solar_system.zodiac_center().xy(time);
        root_drawing_area.fill(&BLACK).unwrap();
        for obj in solar_system.objects() {
            let loc = (obj.xy(time) - offset).loc();
            chart.draw_series(PointSeries::of_element(
                vec![loc],
                stroke_width_base * 5,
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
            chart.draw_series(LineSeries::new(
                vec![(0.0, 0.0), loc],
                Into::<ShapeStyle>::into(obj.get_color()).stroke_width(stroke_width_base)
            )).unwrap();
            // let angle = obj.angle_deg(solar_system.zodiac_center(), time);
            // let angle_rounded = (angle / 30.0).floor() as i32 * 30;
            // let sign = solar_system.zodiac.get(&angle_rounded);
            // println!("{}: {} ({}, {:?})", obj.get_name(), angle, angle_rounded, sign);
            let stroke_width = if obj.get_name() == "Moon" {1} else {2};
            let end_time = match obj.orbital_period(time) {
                Some(op) => time + op,
                None => time + solar_system.zodiac_center().orbital_period(time).unwrap()
            };
            let trajectory: Vec<Point2D> = obj.trajectory_relative(solar_system.zodiac_center(), time, end_time, 100)
                .into_iter().map(|x| x.into()).collect();
            chart.draw_series(LineSeries::new(
                trajectory.into_iter().map(|x| x.loc()),
                Into::<ShapeStyle>::into(obj.get_color()).stroke_width(stroke_width_base * stroke_width),
            )).unwrap();
        }
        root_drawing_area.present().unwrap();
    }
}

pub fn plot_3d<'a>(solar_system: &'a SolarSystem<'a>, pixels: u32, scale: f64, time: f64) {
    let stroke_width_base = (pixels / 2048).max(1);
    let Point3D(ex, ey, ez) = solar_system.zodiac_center().xyz(time);
    println!("Drawing 3d absolute...");

    let root_drawing_area = BitMapBackend::new("images/solar_system_3d.png", (pixels, pixels))
        .into_drawing_area();

    root_drawing_area.fill(&BLACK).unwrap();
    let mut chart = ChartBuilder::on(&root_drawing_area).margin(20).caption("Solar system", ("sans-serif", 20))
        .build_cartesian_3d(
            -scale..scale,
            // -50.0..50.0,
            -scale..scale,
            -scale..scale)
        .unwrap();
    chart.with_projection(|mut pb| {
        pb.pitch = 0.0;
        pb.yaw = 1.0;
        pb.scale = 1.3;
        pb.into_matrix()
    });
    
    chart.configure_axes().draw().unwrap();

    for obj in solar_system.objects() {
        let Point3D(ox, oy, oz) = obj.xyz(time);
        chart.draw_series(PointSeries::of_element(
            vec![(ox, oy, oz)],
            stroke_width_base * 5,
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
        chart.draw_series(LineSeries::new(
            vec![(ex, ey, ez), (ox, oy, oz)],
            Into::<ShapeStyle>::into(obj.get_color()).stroke_width(stroke_width_base)
        )).unwrap();
        let stroke_width = if obj.get_name() == "Moon" {1} else {2};
        let end_time = match obj.orbital_period(time) {
            Some(op) => time + op,
            None => time + 5.0
        };
        let trajectory: Vec<Point3D> = obj.trajectory(time, end_time, 100);
        chart.draw_series(LineSeries::new(
            // (0..11).map(|i| i as f64 * TAU / 10.0).map(|i| match obj.xy(time) { (x, y) => (x + 10.0 * i.cos(), y + 10.0 * i.sin())}),
            trajectory.into_iter().map(|x| x.loc()),
            // (-5000..5000).map(|x| x as f64 / 10.0).map(|x| obj.xy(x)),
            Into::<ShapeStyle>::into(obj.get_color()).stroke_width(stroke_width_base * stroke_width),
        )).unwrap();
    }
}

pub fn plot_rel_3d<'a>(solar_system: &'a SolarSystem<'a>, pixels: u32, scale: f64, time: f64) {
    let stroke_width_base = (pixels / 2048).max(1);
    let offset = solar_system.zodiac_center().xyz(time);
    println!("Drawing 3d relative...");

    let root_drawing_area = BitMapBackend::new("images/solar_system_3d.png", (pixels, pixels))
        .into_drawing_area();

    root_drawing_area.fill(&BLACK).unwrap();
    let mut chart = ChartBuilder::on(&root_drawing_area).margin(20).caption("Solar system", ("sans-serif", 20))
        .build_cartesian_3d(
            -scale..scale,
            // -50.0..50.0,
            -scale..scale,
            -scale..scale)
        .unwrap();
    chart.with_projection(|mut pb| {
        pb.pitch = 0.0;
        pb.yaw = 1.0;
        pb.scale = 1.3;
        pb.into_matrix()
    });
    
    chart.configure_axes().draw().unwrap();

    for obj in solar_system.objects() {
        let loc = (obj.xyz(time) - offset).loc();
        chart.draw_series(PointSeries::of_element(
            vec![loc],
            stroke_width_base * 5,
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
        chart.draw_series(LineSeries::new(
            vec![(0.0, 0.0, 0.0), loc],
            Into::<ShapeStyle>::into(obj.get_color()).stroke_width(stroke_width_base)
        )).unwrap();
        let stroke_width = if obj.get_name() == "Moon" {1} else {2};
        let end_time = match obj.orbital_period(time) {
            Some(op) => time + op,
            None => time + solar_system.zodiac_center().orbital_period(time).unwrap()
        };
        let trajectory: Vec<Point3D> = obj.trajectory_relative(solar_system.zodiac_center(), time, end_time, 100);
        chart.draw_series(LineSeries::new(
            // (0..11).map(|i| i as f64 * TAU / 10.0).map(|i| match obj.xy(time) { (x, y) => (x + 10.0 * i.cos(), y + 10.0 * i.sin())}),
            trajectory.into_iter().map(|x| x.loc()),
            // (-5000..5000).map(|x| x as f64 / 10.0).map(|x| obj.xy(x)),
            Into::<ShapeStyle>::into(obj.get_color()).stroke_width(stroke_width_base * stroke_width),
        )).unwrap();
    }
}



fn main() {
    // let year_seconds = match earth.orbital_period(86400.0 * 365.25 * 10.0) {
    //     Some(t) => t,
    //     None => -1.0,
    // };
    // let year = Duration::seconds(year_seconds as i64);
    // println!("Year: {year}");
    let pixels = 2048;
    let scale = 200.0;
    let solar_system = SolarSystem::new_default();
    let args: Vec<String> = env::args().collect();
    println!("{args:?}");
    if args.len() > 1 {
        let action = &args[1];
        if action == "next" {
            let planet = &args[2];
            let sign = &args[3];
            let time_str = if args.len() >= 5 { &args[4] } else { "now" };
            let start_time = parse_time(time_str).unwrap_or_else(|_| {
                println!("Usage: solar_system next <planet> <sign> [time]"); 
                SystemTime::now().into()
            });
            let sign_time = match solar_system.next_time_in_sign_dt(planet, sign, start_time) {
                Some(st) => st,
                None => { println!("Error: Could not get next sign"); J2000 }
            };
            println!("The next time {planet} will be in {sign} after {start_time} is {sign_time}");
            // solar_system.plot_2d(&earth, dt_to_internal(sign_time));
        }
        else if action == "plot" {
            let dimensions = &args[2];
            let pers = &args[3];
            match (dimensions.as_str(), pers.as_str()) {
                ("2d", "abs") => plot_2d(&solar_system, pixels, scale, 0.0),
                ("2d", "rel") => plot_rel_2d(&solar_system, pixels, scale, 0.0),
                ("3d", "abs") => plot_3d(&solar_system, pixels, scale, 0.0),
                ("3d", "rel") => plot_rel_3d(&solar_system, pixels, scale, 0.0),
                _ => println!("Usage: solar_system plot <2d|3d> <abs|rel>")
            }
        }
        else if action == "sign" {
            let planet = &args[2];
            let time_str = if args.len() >= 4 { args[3].as_str() } else { "now" };
            let time = match parse_time(time_str) {
                Ok(t) => t,
                Err(s) => { println!("time parsing got {s}; using now"); SystemTime::now().into() }
            };
            if planet == "all" {
                println!("Hello world")
            }
            else {
                let sign = solar_system.zodiac_for_dt(planet, time).unwrap();
                println!("Calculating {planet}'s sign as {sign} at {time}");
            }
        }
        else {
            println!("Error: Valid commands are next, sign, and plot")
        }
    }
    else {
        println!("Error: Not enough arguments")
    }
    println!("Done");
}
