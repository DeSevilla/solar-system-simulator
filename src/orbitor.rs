use std::marker::PhantomData;
use std::ops::{Add, Sub};
use std::rc::Rc;
use std::collections::HashMap;
use std::f64::consts::TAU;
use plotters::prelude::*;
use plotters::style::{RGBColor, full_palette::{GREY, PURPLE, BLUE_300, ORANGE, BLUE_100}};
use time::{OffsetDateTime, macros::datetime};
use uom::si::angle::{degree, radian};
use uom::si::mass::kilogram;
use uom::si::ratio::ratio;
use uom::si::time::{day, second};
use uom::si::length::meter;
use uom::si::{f64::*, Quantity};
use uom::typenum::{Z0, P3, N1, N2};

// #[derive(Clone, Copy, Debug)]
// struct Time(f64);

// #[derive(Clone, Copy, Debug)]
// struct Duration(f64);

// impl Add<Duration> for Time {
//     type Output = Self;

//     fn add(self, rhs: Duration) -> Self::Output {
//         Time(self.0 + rhs.0)
//     }
// }

// impl Sub for Time {
//     type Output = Duration;

//     fn sub(self, rhs: Self) -> Self::Output {
//         Duration(self.0 - rhs.0)
//     }
// }

const SCALING_FACTOR: f64 = 25000000000.0;

// /// Convert degrees to radians
// pub fn deg_to_rad(x: f64) -> f64 {
//     x * TAU / 360.0
// }

// /// Convert radians to degrees
// pub fn rad_to_deg(x: f64) -> f64 {
//     x / TAU * 360.0
// }

pub fn normalize(ang: Angle) -> Angle {
    Angle::new::<radian>(ang.get::<radian>().rem_euclid(TAU))
}

pub const J2000: OffsetDateTime = datetime!(2000-01-01 12:00 UTC);

pub fn dt_to_internal(dt: OffsetDateTime) -> Time {
    let (dur, sign) = if J2000 > dt {
        (J2000 - dt, -1.0)
    } 
    else {
        (dt - J2000, 1.0)
    };
    let t: Time = std::time::Duration::try_from(dur).expect("Duration conversion should not violate base type range")
        .try_into().expect("Duration should have been constructed non-negative");
    t * sign
}

pub fn internal_to_dt(time: Time) -> OffsetDateTime {
    let dur = std::time::Duration::try_from(time.abs()).expect("`time.abs()` should be non-negative");
    if time.is_sign_negative() {
        J2000 - dur
    }
    else {
        J2000 + dur
    }
}



// const G: f64 = 6.67430e-11;
const G: Quantity<uom::si::ISQ<P3, N1, N2, Z0, Z0, Z0, Z0>, uom::si::SI<f64>, f64> = Quantity { dimension: PhantomData, units: PhantomData, value: 6.674e-11, };

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point3D(pub Length, pub Length, pub Length);

impl Point3D {
    pub fn loc(self) -> (f64, f64, f64) {
        let Point3D(x, y, z) = self;
        (x.get::<meter>(), y.get::<meter>(), z.get::<meter>())
    }
}

impl Add for Point3D {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        let Point3D(x, y, z) = self;
        let Point3D(x2, y2, z2) = other;
        Point3D(x+x2, y+y2, z+z2)
    }
}

impl Sub for Point3D {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        let Point3D(x, y, z) = self;
        let Point3D(x2, y2, z2) = other;
        Point3D(x-x2, y-y2, z-z2)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point2D(pub Length, pub Length);

impl Point2D {
    pub fn loc(self) -> (f64, f64) {
        let Point2D(x, y) = self;
        (x.get::<meter>(), y.get::<meter>())
    }
}

impl Add for Point2D {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        let Point2D(x, y) = self;
        let Point2D(x2, y2) = other;
        Point2D(x+x2, y+y2)
    }
}

impl Sub for Point2D {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        let Point2D(x, y) = self;
        let Point2D(x2, y2) = other;
        Point2D(x-x2, y-y2)
    }
}

impl From<Point3D> for Point2D {
    fn from(Point3D(x, _, z): Point3D) -> Point2D {
        Point2D(x, z)
    }
}

pub trait Locatable {
    fn xyz(&self, time: Time) -> Point3D;
    fn xy(&self, time: Time) -> Point2D;
    fn angle(&self, other: &impl Locatable, time: Time) -> Angle {
        let Point2D(x, y) = self.xy(time);
        let Point2D(x2, y2) = other.xy(time);
        // ((y - y2).atan2(x - x2).get::<radian>() + TAU) % TAU
        normalize((y - y2).atan2(x - x2))
    }
    // fn angle_rad(&self, other: &impl Locatable, time: Time) -> f64 {
    //     let Point2D(x, y) = self.xy(time);
    //     let Point2D(x2, y2) = other.xy(time);
    //     ((y - y2).atan2(x - x2).get::<radian>() + TAU) % TAU
    //     // (y - y2).atan2(x - x2)
    // }
    // fn angle_deg(&self, other: &impl Locatable, time: Time) -> f64 {
    //     rad_to_deg(self.angle_rad(other, time))
    // }
}

pub struct StaticObject {
    mass: Mass,
    x: Length,
    y: Length,
    z: Length
}

impl StaticObject {
    pub fn new(mass: Mass, x: Length, y: Length, z: Length) -> StaticObject {
        StaticObject {
            mass,
            x,
            y,
            z,
        }
    }
}

pub struct Orbitor {
    mass: Mass,
    parent: Rc<SolarSystemObject>,
    semimajor: Length,
    eccentricity: f64,
    inclination: Angle,
    lan: Angle, //longitude of the ascending node
    aop: Angle, //argument of periapsis
    mae: Angle, //mean anomaly at epoch
}

impl Orbitor {
    pub fn new<T>(
        mass: f64,
        parent: Rc<SolarSystemObject>, 
        semimajor: f64, 
        eccentricity: f64, 
        inclination: f64,
        lan: f64,
        aop: f64,
        mae: f64) -> Orbitor
            where T: uom::si::angle::Unit + uom::si::angle::Conversion<f64> {
        Orbitor {
            mass: Mass::new::<kilogram>(mass),
            parent,
            semimajor: Length::new::<meter>(semimajor),
            eccentricity,
            inclination: Angle::new::<T>(inclination),
            lan: Angle::new::<T>(lan),
            aop: Angle::new::<T>(aop),
            mae: Angle::new::<T>(mae),
        }
    }

    pub fn orbital_period(&self, time: Time) -> Time {
        match self.parent.orbital_period(time) {
            Some(period) => period,
            None => {
                let mu = G * (self.mass + self.parent.get_mass());
                TAU / (mu/ (self.semimajor * self.semimajor * self.semimajor)).sqrt()
            }
        }
    }

    pub fn current_mean_anomaly(&self, time: Time) -> Angle {
        if self.semimajor.get::<meter>() == 0.0 {
            Angle::new::<radian>(0.0)
        }
        else if time == Time::new::<second>(0.0) {
            self.mae
        }
        else {
            let mu = G * (self.mass + self.parent.get_mass());
            let orbital_fraction = time * (mu/(self.semimajor * self.semimajor * self.semimajor)).sqrt();
            let angle = normalize(Angle::from(orbital_fraction));
            normalize(self.mae + angle) //time.get::<second>() * (mu/(self.semimajor * self.semimajor * self.semimajor).sqrt())) //% TAU
        }
    }

    pub fn eccentric_anomaly(&self, mean_anomaly: Angle) -> Angle {
        let mut ecc = mean_anomaly.get::<radian>();
        for _ in 0..5 {
            ecc = ecc - (ecc - self.eccentricity * ecc.sin() - mean_anomaly.get::<radian>())/(1.0 - self.eccentricity * ecc.cos());
        }
        normalize(Angle::new::<radian>(ecc))
    }
    
    pub fn true_anomaly(&self, eccentric_anomaly: Angle) -> Angle {
        let left_term = (1.0 + self.eccentricity).sqrt() * (eccentric_anomaly/2.0).sin();
        let right_term = (1.0 - self.eccentricity).sqrt() * (eccentric_anomaly/2.0).cos();
        normalize(2.0 * left_term.atan2(right_term))
    }
    
    pub fn orbit_xy(&self, time: Time) -> Point2D {
        let mean_anom = self.current_mean_anomaly(time);
        let ecc_anom = self.eccentric_anomaly(mean_anom);
        let true_anom = self.true_anomaly(ecc_anom);
        let radius = self.semimajor * (1.0 - self.eccentricity * ecc_anom.cos().get::<ratio>()); //TODO
        Point2D(radius * true_anom.cos(), radius * true_anom.sin())
    }
    
    pub fn in_parent_coordinates(&self, orbit_loc: Point2D) -> Point3D {
        let Point2D(ox, oy) = orbit_loc;
        let aopcos = self.aop.cos();
        let aopsin = self.aop.sin();
        let lancos = self.lan.cos();
        let lansin = self.lan.sin();
        let inccos = self.inclination.cos();
        let incsin = self.inclination.sin();
        let x = ox * (aopcos * lancos - aopsin * inccos * lansin) - oy * (aopsin * lancos + aopcos * inccos * lansin);
        let y = ox * (aopcos * lansin + aopsin * inccos * lancos) + oy * (aopcos * inccos * lancos - aopsin * lansin);
        let z = ox * aopsin * incsin + oy * aopcos * incsin;
        Point3D(x, z, y)
    }
}

pub enum SolarSystemObject {
    Static { name: String, color: RGBColor, s: StaticObject },
    Orbit { name: String, color: RGBColor, o: Orbitor },
    // Variable { name: String, color: RGBColor, f: & dyn Fn(f64) -> Orbitor}
}

impl SolarSystemObject {
    pub fn new_static(name: &str, color: RGBColor, mass: f64, x: f64, y: f64, z: f64) -> SolarSystemObject {
        SolarSystemObject::Static {
            name: name.into(),
            color,
            s: StaticObject::new(Mass::new::<kilogram>(mass), Length::new::<meter>(x), Length::new::<meter>(y), Length::new::<meter>(z))
        }
    }

    pub fn new_orbitor(name: &str,
                       color: RGBColor,
                       mass: f64,
                       parent: Rc<SolarSystemObject>,
                       semimajor: f64,
                       eccentricity: f64,
                       inclination: f64,
                       lan: f64,
                       aop: f64,
                       mae: f64
                    ) -> SolarSystemObject {
        SolarSystemObject::Orbit {
            name: name.into(),
            color,
            o: Orbitor::new::<degree>(mass, parent, semimajor, eccentricity, inclination, lan, aop, mae)
        }
    }

    // pub fn new_variable(name: &str, color: RGBColor, function: &dyn Fn(f64) -> Orbitor) -> SolarSystemObject {
    //     SolarSystemObject::Variable { 
    //         name: name.into(), 
    //         color, 
    //         f: function 
    //     }
    // }

    pub fn get_name(&self) -> String {
        match self {
            Self::Static { name, .. } => name.clone(),
            Self::Orbit { name, .. } => name.clone(),
            // Self::Variable { name, .. } => name.clone(),
        }
    }

    pub fn get_color(&self) -> RGBColor {
        match self {
            Self::Static { color, .. } => *color,
            Self::Orbit { color, .. } => *color,
            // Self::Variable { color, .. } => *color,
        }
    }

    pub fn get_mass(&self) -> Mass {
        match self {
            Self::Static { s, .. } => s.mass,
            Self::Orbit { o, .. } => o.mass,
            // Self::Variable { f, .. } => f(0.0).mass,
        }
    }

    pub fn orbital_period(&self, start_time: Time) -> Option<Time> {
        match self {
            Self::Orbit { o, .. } => Some (o.orbital_period(start_time)),
            Self::Static { .. } => None,
            // Self::Variable { f, .. } => Some (f(start_time).orbital_period(start_time)),
        }
    }

    pub fn next_time_angle_in_range(&self, other: &SolarSystemObject,
                                    angle_start: Angle, angle_end: Angle,
                                    start_time: Time) -> Option<Time> {
        // TODO there has to be a more efficient way to calculate this...
        let max_time = {
            let t1 = self.orbital_period(start_time).unwrap_or(Time::new::<second>(1.0));
            let t2 = other.orbital_period(start_time).unwrap_or(Time::new::<second>(1.0));
            t1 * t2.get::<second>()
        };
        let mut prev_time = start_time;
        for ii in 0..=(max_time.get::<day>()) as i32 {
            let time = start_time + Time::new::<day>(ii as f64);
            let angle = other.angle(self, time);
            if angle_start < angle && angle < angle_end {
                for jj in 0..(time - prev_time).get::<second>() as i32 {
                    let small_time = prev_time + Time::new::<second>(jj as f64);
                    let small_angle = other.angle(self, small_time);
                    if angle_start < small_angle && small_angle < angle_end {
                        return Some(small_time);
                    }
                }
                return Some(time);
            }
            prev_time = time;
        }
        None
    }

    // pub fn next_time_angle_deg_in_range(&self, other: &SolarSystemObject,
    //                                 angle_start: Angle, angle_end: Angle,
    //                                 start_time: Time) -> Option<Time> {
    //     self.next_time_angle_rad_in_range(
    //         other,
    //         angle_start, angle_end, 
    //         start_time
    //     )
    // }

    pub fn trajectory(&self, start_time: Time, end_time: Time, num_points: i32) -> Vec<Point3D> {
        let mut output = Vec::new();
        let time_range = end_time - start_time;
        for time in (0..=num_points).map(|x| x as f64 * time_range / num_points as f64 + start_time) {
            output.push(self.xyz(time));
        }
        output
    }

    pub fn trajectory_relative(&self, other: &SolarSystemObject, start_time: Time, end_time: Time, num_points: i32) -> Vec<Point3D> {
        let mut output = Vec::new();
        let time_range = end_time - start_time;
        for time in (0..=num_points).map(|x| x as f64 * time_range / num_points as f64 + start_time) {
            output.push(self.xyz(time) - other.xyz(time));
        }
        output
    }
}

pub struct Zodiac {
    signs: Vec<String>,
    // count: usize,
}

impl Zodiac {
    pub fn new(signs: Vec<String>) -> Zodiac {
        // let mut signs = HashMap::new();
        // let count = signs.len();
        // for (i, sign) in sign_names.into_iter().enumerate() {
        //     signs.insert(sign, (i * 360) as f64 / count as f64);
        // }
        Zodiac {
            signs,
            // count: signs.len()
        }
    }

    fn angle_to_index(&self, angle: Angle) -> usize {
        let angle = normalize(angle).get::<degree>();
        (angle * self.signs.len() as f64 / 360.0).floor() as usize
    }

    fn index_to_angle(&self, idx: usize) -> Angle {
        let idx = idx % self.signs.len();
        normalize(Angle::new::<degree>((idx * 360) as f64 / self.signs.len() as f64))
    }

    pub fn angles(&self) -> Vec<Angle> {
        let mut angles = Vec::new();
        for i in 0..self.signs.len() {
            angles.push(self.index_to_angle(i))
        }
        angles
    }

    pub fn get_sign(&self, angle: Angle) -> &String {
        &self.signs[self.angle_to_index(angle)]
    }

    pub fn get_angle_range(&self, sign: String) -> Option<(Angle, Angle)> {
        for (i, s) in self.signs.iter().enumerate() {
            if sign == *s {
                return Some((self.index_to_angle(i), self.index_to_angle(i+1)))
            }
        }
        None
    }
}

pub struct SolarSystem {
    objects: Vec<Rc<SolarSystemObject>>,
    index: HashMap<String, usize>,
    zodiac: Zodiac,
    zodiac_center: usize,
}

impl SolarSystem {
    pub fn new_custom(zodiac_signs: Vec<String>, objects: Vec<Rc<SolarSystemObject>>, zodiac_center: usize) -> SolarSystem {
        let mut index = HashMap::new();
        for (i, obj) in objects.iter().enumerate() {
            index.insert(obj.get_name().to_lowercase(), i);
        };
        SolarSystem {
            objects,
            index,
            zodiac: Zodiac::new(zodiac_signs),
            zodiac_center
        }
    }

    pub fn new_default() -> SolarSystem {
        let zodiac = Zodiac::new(vec![
            "aries".into(),
            "taurus".into(),
            "gemini".into(),
            "cancer".into(),
            "leo".into(),
            "virgo".into(),
            "libra".into(),
            "scorpio".into(),
            "sagittarius".into(),
            "capricorn".into(),
            "aquarius".into(),
            "pisces".into(),
        ]);
        let sun = SolarSystemObject::new_static(
            "Sun", 
            YELLOW,
            1.9885e30,
            0.0,
            0.0,
            0.0);
        let sun_rc = Rc::new(sun);
        let mercury = SolarSystemObject::new_orbitor(
            "Mercury",
            WHITE,
            3.3011e23,
            sun_rc.clone(),
            5.7909e+10,
            0.2056,
            7.004,
            48.331,
            29.124,
            174.796,
        );
        let venus = SolarSystemObject::new_orbitor(
            "Venus",
            PURPLE,
            4.8675e24,
            sun_rc.clone(),
            108208000000.0,
            0.06772,
            3.39458,
            76.68,
            54.884,
            50.115,
        );
        let earth = SolarSystemObject::new_orbitor(
            "Earth",
            BLUE_300,
            5.97217e24,
            sun_rc.clone(),
            149598023000.0,
            0.0167086,
            0.00005,
            -11.26064, 
            114.20783,
            358.617,
        );
        let earth_rc = Rc::new(earth);
        let moon = SolarSystemObject::new_orbitor(
            "Moon",
            GREY,
            7.342e22,
            earth_rc.clone(),
            384399000.0,
            0.0549,
            5.145,
            125.08 - 0.0 * 360.0 / (18.61 * 1000.0),
            318.15 + 0.0 * 360.0 / (8.85 * 1000.0),
            13.13,
        );
        let mars = SolarSystemObject::new_orbitor(
            "Mars",
            RED,
            6.4171e23,
            sun_rc.clone(),
            227939366000.0,
            0.0934,
            1.850,
            49.57854,
            286.5,
            19.412,
        );
        let jupiter = SolarSystemObject::new_orbitor(
            "Jupiter",
            ORANGE,
            1.8982e27,
            sun_rc.clone(),
            778479000000.0,
            0.0489,
            1.303,
            100.464,
            273.867,
            20.020,
        );
        let saturn = SolarSystemObject::new_orbitor(
            "Saturn",
            RGBColor(100, 100, 0),
            5.6834e24,
            sun_rc.clone(),
            1433530000000.0,
            0.0565,
            2.485,
            113.665,
            339.392,
            317.020,
        );
        let uranus = SolarSystemObject::new_orbitor(
            "Uranus",
            BLUE_100,
            8.6810e25,
            sun_rc.clone(),
            2.870972e12,
            0.04717,
            0.773,
            74.006, 
            96.998857,
            142.2386,
        );
        let neptune = SolarSystemObject::new_orbitor(
            "Neptune", 
            BLUE,
            1.02409e26,
            sun_rc.clone(),
            4.514953e12,
            0.008678,
            1.77,
            131.721,
            273.24966,
            256.228,
        );
        let mut solar_system = SolarSystem {
            objects: Vec::new(),
            index: HashMap::new(),
            zodiac,
            zodiac_center: 3,
        };
        solar_system.add(sun_rc);
        solar_system.add(Rc::new(mercury));
        solar_system.add(Rc::new(venus));
        solar_system.add(earth_rc);
        solar_system.add(Rc::new(moon));
        solar_system.add(Rc::new(mars));
        solar_system.add(Rc::new(jupiter));
        solar_system.add(Rc::new(saturn));
        solar_system.add(Rc::new(uranus));
        solar_system.add(Rc::new(neptune));
        solar_system
    }

    pub fn add(&mut self, obj: Rc<SolarSystemObject>) {
        self.index.insert(obj.get_name().to_lowercase(), self.objects.len());
        self.objects.push(obj);
    }

    pub fn objects(&self) -> &Vec<Rc<SolarSystemObject>> {
        &(self.objects)
    }

    pub fn names(&self) -> Vec<String> {
        self.objects.iter()
            .map(|x| x.get_name())
            // .filter(|s| !zodiac_only || s != &solar_system.zodiac_center().get_name())
            .collect()
    }

    pub fn zodiac(&self) -> &Zodiac {
        &self.zodiac
    }

    pub fn get(&self, obj_name: &str) -> Option<&SolarSystemObject> {
        // let string_name = obj_name.into();
        let obj_idx = self.index.get(&obj_name.to_lowercase())?;
        let obj = self.objects.get(*obj_idx)?;
        Some(obj)
    }

    pub fn zodiac_center(&self) -> &SolarSystemObject {
        &self.objects[self.zodiac_center]
    }

    pub fn angle_to_sign(&self, angle: Angle) -> String {
        self.zodiac.get_sign(angle).clone()
    }

    pub fn next_time_in_sign_dt(&self, 
                                obj_name: &str, 
                                sign_name: &str, 
                                start_time: OffsetDateTime) -> Option<OffsetDateTime> {
        let start = dt_to_internal(start_time);
        let next = self.next_time_in_sign(obj_name, sign_name, start)?;
        Some(internal_to_dt(next))
    }

    pub fn next_time_in_sign(&self, 
                            obj_name: &str,
                            sign_name: &str, 
                            start_time: Time) -> Option<Time> {
        let string_sign = sign_name.to_lowercase();
        let (angle_start, angle_end) = self.zodiac.get_angle_range(string_sign)?;
        let obj = self.get(obj_name)?;
        self.zodiac_center().next_time_angle_in_range(
            obj,
            angle_start,
            angle_end,
            start_time
        )
    }

    pub fn zodiac_for_dt(&self, obj_name: &str, time: OffsetDateTime) -> Option<String> {
        let t = dt_to_internal(time);
        self.zodiac_for(obj_name, t)
    }

    pub fn zodiac_for(&self, obj_name: &str, time: Time) -> Option<String> {
        let obj = self.get(obj_name)?;
        let angle = obj.angle(self.zodiac_center(), time);
        Some(self.angle_to_sign(angle))
    }
}

impl Locatable for Orbitor {
    fn xyz(&self, time: Time) -> Point3D {
        let Point3D(x, y, z) = self.parent.xyz(time);
        let Point3D(x2, y2, z2) = self.in_parent_coordinates(self.orbit_xy(time));
        Point3D(x + x2 / SCALING_FACTOR, y + y2 / SCALING_FACTOR, z + z2 / SCALING_FACTOR)
    }

    fn xy(&self, time: Time) -> Point2D {
        let Point3D(x, _, z) = self.xyz(time);
        // println!("{x} {y}");
        Point2D(x, z)
    }
}

impl Locatable for StaticObject {
    fn xyz(&self, _time: Time) -> Point3D {
        Point3D(self.x, self.y, self.z)
    }

    fn xy(&self, _time: Time) -> Point2D {
        Point2D(self.x, self.z)
    }
}

impl Locatable for SolarSystemObject {
    fn xyz(&self, time: Time) -> Point3D {
        match self {
            Self::Static { s, .. } => s.xyz(time),
            Self::Orbit { o, .. } => o.xyz(time),
            // Self::Variable { f, .. } => f(time).xyz(time),
        }
    }

    fn xy(&self, time: Time) -> Point2D {
        match self {
            Self::Static { s, .. } => s.xy(time),
            Self::Orbit { o, .. } => o.xy(time),
            // Self::Variable { f, .. } => f(time).xy(time),
        }
    }

}
