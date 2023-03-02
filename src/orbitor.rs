pub mod orbitor {
    use std::ops::{Add, Sub};
    use std::{collections::HashMap};
    use std::{f64::consts::TAU};
    use plotters::{prelude::*, style::RGBColor, style::full_palette::{GREY}};
    use time::{OffsetDateTime, macros::datetime};


    const SCALING_FACTOR: f64 = 25000000.0;

    pub fn deg_to_rad(x: f64) -> f64 {
        x * TAU / 360.0
    }

    pub fn rad_to_deg(x: f64) -> f64 {
        x / TAU * 360.0
    }

    pub const J2000: OffsetDateTime = datetime!(2000-01-01 12:00 UTC);

    pub fn convert_datetime(dt: OffsetDateTime) -> f64 {
        (dt.to_julian_day() - J2000.to_julian_day()) as f64 / 365.25 * 997.94  
        // 997.94 is what i get for earth's orbit
        // probably wrong but we're not really trying to be super rigorous
        // don't even have most perturbations in
    }

    const G: f64 = 6.67430e-11;

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct Point2D(f64, f64);

    impl Point2D {
        pub fn loc(self) -> (f64, f64) {
            let Point2D(x, y) = self;
            (x, y)
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

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct Point3D(f64, f64, f64);

    impl Point3D {
        pub fn loc(self) -> (f64, f64, f64) {
            let Point3D(x, y, z) = self;
            (x, y, z)
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

    pub trait Locatable {
        fn xyz(&self, time: f64) -> Point3D;
        fn xy(&self, time: f64) -> Point2D;
        fn angle_rad(&self, other: &dyn Locatable, time: f64) -> f64 {
            let Point2D(x, y) = self.xy(time);
            let Point2D(x2, y2) = other.xy(time);
            ((y - y2).atan2(x - x2) + TAU) % TAU
        }
        fn angle_deg(&self, other: &dyn Locatable, time: f64) -> f64 {
            rad_to_deg(self.angle_rad(other, time))
        }
    }

    pub struct StaticObject {
        mass: f64,
        x: f64,
        y: f64,
        z: f64
    }
    
    impl StaticObject {
        pub fn new(mass: f64, x: f64, y: f64, z:f64) -> StaticObject {
            StaticObject {
                mass: mass,
                x: x,
                y: y,
                z: z,
            }
        }
    }

    pub struct Orbitor<'a> {
        mass: f64,
        parent: &'a SolarSystemObject<'a>,
        semimajor: f64,
        eccentricity: f64,
        inclination: f64,
        lan: f64, //longitude of the ascending node
        aop: f64, //argument of periapsis
        mae: f64, //mean anomaly at epoch
    }

    impl<'a> Orbitor<'a> {
        pub fn new(
            mass: f64,
            parent: &'a SolarSystemObject<'a>, 
            semimajor: f64, 
            eccentricity: f64, 
            inclination: f64, 
            lan: f64,
            aop: f64,
            mae: f64) -> Orbitor<'a> {
            Orbitor {
                mass: mass,
                parent: parent,
                semimajor: semimajor,
                eccentricity: eccentricity,
                inclination: inclination,
                lan: lan,
                aop: aop,
                mae: mae,
            }
        }

        fn orbital_period(&self, time: f64) -> f64 {
            match self.parent.orbital_period(time) {
                Some(period) => period,
                None => {
                    let mu = G * (self.mass + self.parent.get_mass());
                    TAU / (mu/self.semimajor.powi(3)).sqrt()
                }
            }
        }

        fn current_mean_anomaly(&self, time: f64) -> f64 {
            if self.semimajor == 0.0 {
                0.0
            }
            else if time == 0.0 {
                self.mae
            }
            else {
                let mu = G * (self.mass + self.parent.get_mass());
                (self.mae + time * (mu/self.semimajor.powi(3)).sqrt()) % TAU
            }
        }

        fn eccentric_anomaly(&self, mean_anomaly: f64) -> f64 {
            let mut ecc = mean_anomaly;
            for _ in 0..5 {
                ecc = ecc - (ecc - self.eccentricity * ecc.sin() - mean_anomaly)/(1.0 - self.eccentricity * ecc.cos());
            }
            // println!("error: {}", mean_anomaly - ecc + self.eccentricity * ecc.sin());
            // println!("mean_anom: {mean_anomaly} ecc: {ecc}");
            ecc
        }
        
        fn true_anomaly(&self, eccentric_anomaly: f64) -> f64 {
            let left_term = (1.0 + self.eccentricity).sqrt() * (eccentric_anomaly/2.0).sin();
            let right_term = (1.0 - self.eccentricity).sqrt() * (eccentric_anomaly/2.0).cos();
            2.0 * left_term.atan2(right_term)
        }
        
        fn orbit_xy(&self, time: f64) -> Point2D {
            let mean_anom = self.current_mean_anomaly(time);
            let ecc_anom = self.eccentric_anomaly(mean_anom);
            let true_anom = self.true_anomaly(ecc_anom);
            // println!("err {}", mean_anom - true_anom);
            let radius = self.semimajor * (1.0 - self.eccentricity * ecc_anom.cos());
            Point2D(radius * true_anom.cos(), radius * true_anom.sin())
        }
        
        fn in_parent_coordinates(&self, orbit_loc: Point2D) -> Point3D {
            let Point2D(ox, oy) = orbit_loc;
            let aopcos = self.aop.cos();
            let aopsin = self.aop.sin();
            let lancos = self.lan.cos();
            let lansin = self.lan.sin();
            let inccos = self.inclination.cos();
            let incsin = self.inclination.sin();
            // println!("{aopcos} {aopsin} {lancos} {lansin} {inccos} {incsin}");
            let x = ox * (aopcos * lancos - aopsin * inccos * lansin) - oy * (aopsin * lancos + aopcos * inccos * lansin);
            let y = ox * (aopcos * lansin + aopsin * inccos * lancos) + oy * (aopcos * inccos * lancos - aopsin * lansin);
            let z = ox * aopsin * incsin + oy * aopcos * incsin;
            Point3D(x, z, y)
        }
    }

    pub enum SolarSystemObject<'a> {
        Static { name: String, color: &'a RGBColor, s: StaticObject },
        Orbit { name: String, color: &'a RGBColor, o: Orbitor<'a> },
        Variable { name: String, color: &'a RGBColor, f: &'a dyn Fn(f64) -> Orbitor<'a>}
    }

    impl<'a> SolarSystemObject<'a> {
        pub fn new_static(name: String, color: &'a RGBColor, mass: f64, x: f64, y: f64, z: f64) -> SolarSystemObject<'a> {
            SolarSystemObject::Static {
                name: name,
                color: color,
                s: StaticObject::new(mass, x, y, z)
            }
        }

        pub fn new_orbitor(name: String,
                           color: &'a RGBColor,
                           mass: f64,
                           parent: &'a SolarSystemObject<'a>,
                           semimajor: f64,
                           eccentricity: f64,
                           inclination: f64,
                           lan: f64,
                           aop: f64,
                           mae: f64
                        ) -> SolarSystemObject<'a> {
            SolarSystemObject::Orbit {
                name: name,
                color: color,
                o: Orbitor::new(mass, parent, semimajor, eccentricity,
                    deg_to_rad(inclination), deg_to_rad(lan), deg_to_rad(aop), deg_to_rad(mae))}
        }

        pub fn new_variable(name: String, color: &'a RGBColor, function: &'a dyn Fn(f64) -> Orbitor<'a>) -> SolarSystemObject<'a> {
            SolarSystemObject::Variable { 
                name: name, 
                color: color, 
                f: function 
            }
        }

        pub fn get_name(&self) -> &String {
            match self {
                Self::Static { name, .. } => name,
                Self::Orbit { name, .. } => name,
                Self::Variable { name, .. } => name,
            }
        }

        pub fn get_color(&self) -> &RGBColor {
            match self {
                Self::Static { color, .. } => color,
                Self::Orbit { color, .. } => color,
                Self::Variable { color, .. } => color,
            }
        }

        pub fn get_mass(&self) -> f64 {
            match self {
                Self::Static { s, .. } => s.mass,
                Self::Orbit { o, .. } => o.mass,
                Self::Variable { f, .. } => f(0.0).mass,
            }
        }

        pub fn orbital_period(&self, start_time: f64) -> Option<f64> {
            match self {
                Self::Orbit { o, .. } => Some (o.orbital_period(start_time)),
                Self::Static { .. } => None,
                Self::Variable { f, .. } => Some (f(start_time).orbital_period(start_time)),
            }
        }

        fn trajectory_2d(&self, start_time: f64, num_points: i32) -> Vec<Point2D> {
            match self {
                Self::Static { s, .. } => vec![s.xy(start_time)],
                _ => {
                    let mut output = Vec::new();
                    let time_range = self.orbital_period(start_time).unwrap();
                    // TODO don't like that unwrap much, but it does work in this case
                    for time in (0..num_points+1).map(|x| x as f64 * time_range / num_points as f64 + start_time) {
                        output.push(self.xy(time));
                    }
                    output
                }
            }
        }

        fn trajectory_3d(&self, start_time: f64, num_points: i32) -> Vec<Point3D> {
            match self {
                Self::Static { s, .. } => vec![s.xyz(0.0)],
                _ => {
                    let mut output = Vec::new();
                    let time_range = self.orbital_period(start_time).unwrap();
                    // TODO don't like that unwrap much, but it does work in this case
                    for time in (0..num_points+1).map(|x| x as f64 * time_range / num_points as f64 + start_time) {
                        output.push(self.xyz(time));
                    }
                    output
                }
            }
        }

        fn trajectory_relative_2d(&self, other: &SolarSystemObject, start_time: f64, num_points: i32) -> Vec<Point2D> {
            match self {
                Self::Static { .. } => {
                    let mut output = Vec::new();
                    match other.orbital_period(start_time) {
                        Some(time_range) => {
                            for time in (0..num_points+1).map(|x| x as f64 * time_range / num_points as f64 + start_time) {
                                output.push(self.xy(time) - other.xy(time));
                            }
                            output
                        }
                        None => vec![self.xy(start_time) - other.xy(start_time)]
                    }
                }
                _ => {
                    let mut output = Vec::new();
                    // TODO we need to figure out what all this covers
                    match other.orbital_period(start_time) {
                        Some(time_range) => {
                            let full_period = time_range.max(self.orbital_period(start_time).unwrap());
                            for time in (0..num_points+1).map(|x| x as f64 * full_period / num_points as f64 + start_time) {
                                output.push(self.xy(time) - other.xy(time));
                            }
                            output
                        }
                        None => vec![self.xy(start_time) - other.xy(start_time)]
                    } 
                }
            }
        }

        fn trajectory_relative_3d(&self, other: &SolarSystemObject, start_time: f64, num_points: i32) -> Vec<Point3D> {
            match self {
                Self::Static { .. } => {
                    let mut output = Vec::new();
                    match other.orbital_period(start_time) {
                        Some(time_range) => {
                            for time in (0..num_points+1).map(|x| x as f64 * time_range / num_points as f64 + start_time) {
                                output.push(self.xyz(time) - other.xyz(time));
                            }
                            output
                        }
                        None => vec![self.xyz(start_time) - other.xyz(start_time)]
                    }
                }
                _ => {
                    let mut output = Vec::new();
                    // TODO we need to figure out what all this covers
                    match other.orbital_period(start_time) {
                        Some(time_range) => {
                            for time in (0..num_points+1).map(|x| x as f64 * time_range / num_points as f64 + start_time) {
                                output.push(self.xyz(time) - other.xyz(time));
                            }
                            output
                        }
                        None => vec![self.xyz(start_time) - other.xyz(start_time)]
                    } 
                }
            }
        }
    }

    pub struct SolarSystem<'a> {
        objects: Vec<&'a SolarSystemObject<'a>>,
        index: HashMap<String, &'a SolarSystemObject<'a>>,
        zodiac: HashMap<i32, String>,
        pub pixels: u32,
        pub stroke_width_base: u32,
        pub scale: f64,
    }

    impl<'a> SolarSystem<'a> {
        pub fn new(pixels: u32, scale: f64) -> SolarSystem<'a> {
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
            SolarSystem {
                objects: Vec::new(),
                index: HashMap::new(),
                zodiac: zodiac,
                pixels: pixels,
                stroke_width_base: (pixels / 2048).max(1),
                scale: scale,
            }
        }

        pub fn add(&mut self, obj: &'a SolarSystemObject<'a>) {
            self.objects.push(obj);
            self.index.insert(obj.get_name().clone(), obj);
        }

        pub fn objects(&'a self) -> &'a Vec<&'a SolarSystemObject<'a>> {
            &(self.objects)
        }

        pub fn plot_2d(&'a self, zodiac_center: &SolarSystemObject, time: f64) {
            
            println!("Drawing...");

            let Point2D(ex, ey) = zodiac_center.xy(time);

            // let root_drawing_area = SVGBackend::new("images/solar_system.svg", (self.pixels, self.pixels))
            //     .into_drawing_area();
            let root_drawing_area = BitMapBackend::new("images/solar_system.png", (self.pixels, self.pixels))
                .into_drawing_area();

            root_drawing_area.fill(&BLACK).unwrap();
            let mut chart = ChartBuilder::on(&root_drawing_area)
                .build_cartesian_2d(-self.scale..self.scale, -self.scale..self.scale)
                .unwrap();

            for angle in self.zodiac.keys() {
                let angle_rad = deg_to_rad(*angle as f64);
                let dx = angle_rad.cos();
                let dy = angle_rad.sin();
                let far_edge = (ex + self.scale * dx, ey + self.scale * dy);
                chart.draw_series(LineSeries::new(
                    vec![(ex, ey), far_edge],
                    Into::<ShapeStyle>::into(&GREY).stroke_width(self.stroke_width_base),
                )).unwrap();
            }
            for obj in self.objects() {
                let Point2D(ox, oy) = obj.xy(time);
                chart.draw_series(PointSeries::of_element(
                    vec![(ox, oy)],
                    self.stroke_width_base * 5,
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
                let angle = obj.angle_deg(zodiac_center, time);
                chart.draw_series(LineSeries::new(
                    vec![(ex, ey), (ox, oy)],
                    Into::<ShapeStyle>::into(obj.get_color()).stroke_width(self.stroke_width_base)
                )).unwrap();
                let angle_rounded = (angle / 30.0).floor() as i32 * 30;
                let sign = self.zodiac.get(&angle_rounded);
                println!("{}: {} ({}, {:?})", obj.get_name(), angle, angle_rounded, sign);
                let stroke_width = if obj.get_name() == "Moon" {1} else {2};
                chart.draw_series(LineSeries::new(
                    obj.trajectory_2d(time, 200).iter().map(|x| x.loc()),
                    Into::<ShapeStyle>::into(obj.get_color()).stroke_width(self.stroke_width_base * stroke_width),
                )).unwrap();
            }
        }

        pub fn plot_rel_2d(&'a self, center: &SolarSystemObject, start_time: f64) {
            
            println!("Drawing...");


            // let root_drawing_area = SVGBackend::new("images/solar_system.svg", (self.pixels, self.pixels))
            //     .into_drawing_area();
            // let root_drawing_area = BitMapBackend::new("images/solar_system.png", (self.pixels, self.pixels))
            let root_drawing_area = BitMapBackend::gif("images/solar_system_anim.gif", (self.pixels, self.pixels), 100).unwrap()
                .into_drawing_area();

            let mut chart = ChartBuilder::on(&root_drawing_area)
                .build_cartesian_2d(-self.scale..self.scale, -self.scale..self.scale)
                .unwrap();

            // for angle in self.zodiac.keys() {
            //     let angle_rad = deg_to_rad(*angle as f64);
            //     let dx = angle_rad.cos();
            //     let dy = angle_rad.sin();
            //     let far_edge = center + Point2D(self.scale * dx, self.scale * dy);
            //     chart.draw_series(LineSeries::new(
            //         vec![center.loc(), far_edge.loc()],
            //         Into::<ShapeStyle>::into(&GREY).stroke_width(self.stroke_width_base),
            //     )).unwrap();
            // }
            for i in 0..400 {
                if i % 10 == 0 {
                    println!("{i}");
                }
                let time = start_time - (i * 5) as f64;
                let offset = center.xy(time);
                root_drawing_area.fill(&BLACK).unwrap();
                for obj in self.objects() {
                    let loc = (obj.xy(time) - offset).loc();
                    chart.draw_series(PointSeries::of_element(
                        vec![loc],
                        self.stroke_width_base * 5,
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
                        Into::<ShapeStyle>::into(obj.get_color()).stroke_width(self.stroke_width_base)
                    )).unwrap();
                    // let angle = obj.angle_deg(center, time);
                    // let angle_rounded = (angle / 30.0).floor() as i32 * 30;
                    // let sign = self.zodiac.get(&angle_rounded);
                    // println!("{}: {} ({}, {:?})", obj.get_name(), angle, angle_rounded, sign);
                    let stroke_width = if obj.get_name() == "Moon" {1} else {2};
                    chart.draw_series(LineSeries::new(
                        obj.trajectory_relative_2d(center, time, 100).iter().map(|x| x.loc()),
                        Into::<ShapeStyle>::into(obj.get_color()).stroke_width(self.stroke_width_base * stroke_width),
                    )).unwrap();
                }
                root_drawing_area.present().unwrap();
            }
        }



        pub fn plot_3d(&'a self, zodiac_center: &SolarSystemObject, time: f64) {
            let Point3D(ex, ey, ez) = zodiac_center.xyz(time);
            println!("Drawing...");

            let root_drawing_area = BitMapBackend::new("images/solar_system_3d.png", (self.pixels, self.pixels))
                .into_drawing_area();

            root_drawing_area.fill(&BLACK).unwrap();
            let mut chart = ChartBuilder::on(&root_drawing_area).margin(20).caption("Solar system", ("sans-serif", 20))
                .build_cartesian_3d(
                    -self.scale..self.scale,
                    // -50.0..50.0,
                    -self.scale..self.scale,
                    -self.scale..self.scale)
                .unwrap();
            chart.with_projection(|mut pb| {
                pb.pitch = 0.0;
                pb.yaw = 1.0;
                pb.scale = 1.3;
                pb.into_matrix()
            });
            
            chart.configure_axes().draw().unwrap();

            for obj in self.objects() {
                let Point3D(ox, oy, oz) = obj.xyz(time);
                chart.draw_series(PointSeries::of_element(
                    vec![(ox, oy, oz)],
                    self.stroke_width_base * 5,
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
                let angle = obj.angle_deg(zodiac_center, time);
                chart.draw_series(LineSeries::new(
                    vec![(ex, ey, ez), (ox, oy, oz)],
                    Into::<ShapeStyle>::into(obj.get_color()).stroke_width(self.stroke_width_base)
                )).unwrap();
                let angle_rounded = (angle / 30.0).floor() as i32 * 30;
                let sign = self.zodiac.get(&angle_rounded);
                println!("{}: {} ({}, {:?})", obj.get_name(), angle, angle_rounded, sign);
                let stroke_width = if obj.get_name() == "Moon" {1} else {2};
                chart.draw_series(LineSeries::new(
                    // (0..11).map(|i| i as f64 * TAU / 10.0).map(|i| match obj.xy(time) { (x, y) => (x + 10.0 * i.cos(), y + 10.0 * i.sin())}),
                    obj.trajectory_3d(time, 45).iter().map(|x| x.loc()),
                    // (-5000..5000).map(|x| x as f64 / 10.0).map(|x| obj.xy(x)),
                    Into::<ShapeStyle>::into(obj.get_color()).stroke_width(self.stroke_width_base * stroke_width),
                )).unwrap();
            }
        }

        pub fn plot_rel_3d(&'a self, center: &SolarSystemObject, time: f64) {
            let offset = center.xyz(time);
            println!("Drawing...");

            let root_drawing_area = BitMapBackend::new("images/solar_system_3d.png", (self.pixels, self.pixels))
                .into_drawing_area();

            root_drawing_area.fill(&BLACK).unwrap();
            let mut chart = ChartBuilder::on(&root_drawing_area).margin(20).caption("Solar system", ("sans-serif", 20))
                .build_cartesian_3d(
                    -self.scale..self.scale,
                    // -50.0..50.0,
                    -self.scale..self.scale,
                    -self.scale..self.scale)
                .unwrap();
            chart.with_projection(|mut pb| {
                pb.pitch = 0.0;
                pb.yaw = 1.0;
                pb.scale = 1.3;
                pb.into_matrix()
            });
            
            chart.configure_axes().draw().unwrap();

            for obj in self.objects() {
                let loc = (obj.xyz(time) - offset).loc();
                chart.draw_series(PointSeries::of_element(
                    vec![loc],
                    self.stroke_width_base * 5,
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
                let angle = obj.angle_deg(center, time);
                chart.draw_series(LineSeries::new(
                    vec![(0.0, 0.0, 0.0), loc],
                    Into::<ShapeStyle>::into(obj.get_color()).stroke_width(self.stroke_width_base)
                )).unwrap();
                let angle_rounded = (angle / 30.0).floor() as i32 * 30;
                let sign = self.zodiac.get(&angle_rounded);
                println!("{}: {} ({}, {:?})", obj.get_name(), angle, angle_rounded, sign);
                let stroke_width = if obj.get_name() == "Moon" {1} else {2};
                chart.draw_series(LineSeries::new(
                    // (0..11).map(|i| i as f64 * TAU / 10.0).map(|i| match obj.xy(time) { (x, y) => (x + 10.0 * i.cos(), y + 10.0 * i.sin())}),
                    obj.trajectory_relative_3d(center, time, 45).iter().map(|x| x.loc()),
                    // (-5000..5000).map(|x| x as f64 / 10.0).map(|x| obj.xy(x)),
                    Into::<ShapeStyle>::into(obj.get_color()).stroke_width(self.stroke_width_base * stroke_width),
                )).unwrap();
            }
        }



    }

    impl Locatable for Orbitor<'_> {
        fn xyz(&self, time: f64) -> Point3D {
            let Point3D(x, y, z) = self.parent.xyz(time);
            let Point3D(x2, y2, z2) = self.in_parent_coordinates(self.orbit_xy(time));
            Point3D(x + x2 / SCALING_FACTOR, y + y2 / SCALING_FACTOR, z + z2 / SCALING_FACTOR)
        }

        fn xy(&self, time: f64) -> Point2D {
            let Point3D(x, _, z) = self.xyz(time);
            // println!("{x} {y}");
            Point2D(x, z)
        }
    }

    impl Locatable for StaticObject {
        fn xyz(&self, _time: f64) -> Point3D {
            Point3D(self.x, self.y, self.z)
        }

        fn xy(&self, _time: f64) -> Point2D {
            Point2D(self.x, self.z)
        }

    }

    impl Locatable for SolarSystemObject<'_> {
        fn xyz(&self, time: f64) -> Point3D {
            match self {
                Self::Static { s, .. } => s.xyz(time),
                Self::Orbit { o, .. } => o.xyz(time),
                Self::Variable { f, .. } => f(time).xyz(time),
            }
        }

        fn xy(&self, time: f64) -> Point2D {
            match self {
                Self::Static { s, .. } => s.xy(time),
                Self::Orbit { o, .. } => o.xy(time),
                Self::Variable { f, .. } => f(time).xy(time),
            }
        }

    }
}

