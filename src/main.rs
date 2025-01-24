use std::time::SystemTime;
use orbitor::dt_to_internal;
use time::{
    OffsetDateTime, 
    format_description::well_known::{Iso8601, Rfc3339, Rfc2822}, 
    Date, 
    macros::format_description
};
use clap::*;
use plotters::{prelude::*,  style::full_palette::GREY};

mod orbitor;

use crate::orbitor::{
    SolarSystem,
    Locatable,
    // J2000, 
    Point2D, Point3D,
    deg_to_rad,
};

fn parse_time(time_str: &str) -> Result<OffsetDateTime, String> {
    if let Ok(time) = OffsetDateTime::parse(time_str, &Iso8601::DEFAULT) {
        Ok(time)
    } else if let Ok(time) = OffsetDateTime::parse(time_str, &Rfc2822) {
        Ok(time)
    } else if let Ok(time) = OffsetDateTime::parse(time_str, &Rfc3339) {
        Ok(time)
    } else if let Ok(date) = Date::parse(time_str, format_description!("[year]-[month]-[day]")) {
        Ok(date.midnight().assume_utc())
    } else if time_str == "now" {
        Ok(SystemTime::now().into())
    } else {
        Err("Unknown".into())
    }
}

pub fn plot_2d(solar_system: &SolarSystem, pixels: u32, scale: f64, time: f64) {
    let stroke_width_base = (pixels / 2048).max(1);
    
    println!("Drawing 2d absolute...");

    let Point2D(ex, ey) = solar_system.zodiac_center().xy(time);

    // let root_drawing_area = SVGBackend::new("images/solar_system.svg", (pixels, pixels))
    //     .into_drawing_area();
    let root_drawing_area = BitMapBackend::new("images/solar_system.png", (pixels, pixels))
        .into_drawing_area();

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
            Into::<ShapeStyle>::into(GREY).stroke_width(stroke_width_base),
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

pub fn plot_rel_2d(solar_system: &SolarSystem, pixels: u32, scale: f64, start_time: f64) {
    
    let stroke_width_base = (pixels / 2048).max(1);
    println!("Drawing 2d relative...");

    // let root_drawing_area = SVGBackend::new("images/solar_system.svg", (pixels, pixels))
    let root_drawing_area = BitMapBackend::new("images/solar_system.png", (pixels, pixels))
    // let root_drawing_area = BitMapBackend::gif("images/solar_system_anim.gif", (pixels, pixels), 100).unwrap()
        .into_drawing_area();

    let mut chart = ChartBuilder::on(&root_drawing_area)
        .build_cartesian_2d(-scale..scale, -scale..scale)
        .unwrap();

    for angle in solar_system.zodiac().angles() {
        let angle_rad = deg_to_rad(angle);
        let dx = angle_rad.cos();
        let dy = angle_rad.sin();
        let far_edge = solar_system.zodiac_center().xy(start_time) + Point2D(scale * dx, scale * dy);
        chart.draw_series(LineSeries::new(
            vec![solar_system.zodiac_center().xy(start_time).loc(), far_edge.loc()],
            Into::<ShapeStyle>::into(GREY).stroke_width(stroke_width_base),
        )).unwrap();
    }
    for i in 0..400 {
        // if i % 10 == 0 {
        //     println!("{i}");
        // }
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

pub fn plot_3d(solar_system: &SolarSystem, pixels: u32, scale: f64, time: f64) {
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
        pb.pitch = 0.1;
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

pub fn plot_rel_3d(solar_system: &SolarSystem, pixels: u32, scale: f64, time: f64) {
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

fn print_next_sign_time(solar_system: &SolarSystem, planets: Vec<String>, sign: &String, start_time: OffsetDateTime) {
    println!("Calculating next time for {sign} starting from {start_time}:");
    for planet in planets {
        // println!("Starting {planet}");
        match solar_system.zodiac_for_dt(&planet, start_time) {
            Some(s) => if s == sign.to_lowercase() {
                    println!("  {planet}: already in {sign} at {start_time}");
                } else {
                    // println!("{planet} starts in {s}");
                    match solar_system.next_time_in_sign_dt(&planet, sign, start_time) {
                        Some(st) => println!("  {planet}: {st}"),
                        None => println!("Error: Could not get next time {planet} will be in {sign}"),
                    };
                },
            None => println!("Invalid")
        }
    }
}

fn print_current_signs(solar_system: &SolarSystem, planets: Vec<String>, time: OffsetDateTime) {
    println!("Signs at {time}:");
    for name in planets {
        let sign = match solar_system.zodiac_for_dt(&name, time) {
            Some(s) => s,
            None => "Invalid".to_owned(),
        };
        println!("  {name}: {sign}");
    }
}

#[derive(ValueEnum, Debug, Clone, Copy)]
enum ZodiacObject {
    Sun,
    Mercury,
    Venus,
    Moon,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ZodiacSign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum PlotMode {
    Abs2d,
    Rel2d,
    Abs3d,
    Rel3d,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    Plot {
        // #[arg(short, long, value_delimiter=',', num_args=1..)]
        // objects: Option<Vec<PredefinedObject>>,
        #[arg(short, long, default_value="2048")]
        pixels: u32,
        #[arg(short, long, default_value="200.0")]
        scale: f64,
        #[arg(short, long, default_value="now", value_parser=parse_time)]
        time: OffsetDateTime,
        #[arg(short, long, default_value="abs2d")]
        mode: PlotMode,
    },
    Sign { 
        #[arg(short, long, value_delimiter=',', num_args=1..)]
        planets: Option<Vec<ZodiacObject>>,
        #[arg(short, long, default_value="now", value_parser=parse_time)]
        time: OffsetDateTime,
    },
    Next { 
        sign: ZodiacSign,
        #[arg(short, long, value_delimiter=',', num_args=1..)]
        planets: Option<Vec<ZodiacObject>>,
        #[arg(short, long, default_value="now", value_parser=parse_time)]
        time: OffsetDateTime,
    },
}

//Keplerian simulation of the solar system
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    // #[arg(short, long, default_value="2048")]
    // pixels: u32,
    // #[arg(short, long, default_value="200.0")]
    // scale: f64,
    // #[arg(short, long, value_delimiter=',', num_args=1..)]
    // objects: Option<Vec<PredefinedObject>>,
    #[command(subcommand)]
    command: Command,
}

fn main() {
    let solar_system = SolarSystem::new_default();
    let args = Args::parse();
    match args.command {
        Command::Plot { pixels, scale, time, mode, } => {
            let start_time = dt_to_internal(time);
            match mode {
                PlotMode::Abs2d => plot_2d(&solar_system, pixels, scale, start_time),
                PlotMode::Rel2d => plot_rel_2d(&solar_system, pixels, scale, start_time),
                PlotMode::Abs3d => plot_3d(&solar_system, pixels, scale, start_time),
                PlotMode::Rel3d => plot_rel_3d(&solar_system, pixels, scale, start_time),
            }
        },
        Command::Sign { planets, time } => {
            let planet_names = match planets {
                Some(pl) => pl.iter().map(|x| format!("{x:?}")).collect(),
                None => solar_system.names().into_iter().filter(|s| *s != solar_system.zodiac_center().get_name()).collect(),
            };
            print_current_signs(&solar_system, planet_names, time);
        },
        Command::Next { sign, planets, time } => {
            let planet_names = match planets {
                Some(pl) => pl.iter().map(|x| format!("{x:?}")).collect(),
                None => solar_system.names().into_iter().filter(|s| *s != solar_system.zodiac_center().get_name()).collect(),
            };
            print_next_sign_time(&solar_system, planet_names, &format!("{sign:?}"), time);
        }
    }
}
