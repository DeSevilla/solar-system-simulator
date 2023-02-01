pub mod orbitor {
    use std::f64::consts::TAU;
    use plotters::{style::RGBColor};

    const SCALING_FACTOR: f64 = 25000000.0;

    pub fn deg_to_rad(x: f64) -> f64 {
        x * TAU / 360.0
    }

    pub fn rad_to_deg(x: f64) -> f64 {
        x / TAU * 360.0
    }

    const G: f64 = 6.67430e-11;

    pub type Point2D = (f64, f64);
    pub type Point3D = (f64, f64, f64);

    pub trait Locatable {
        fn xyz(&self, time: f64) -> Point3D;
        fn xy(&self, time: f64) -> Point2D;
        fn trajectory_2d(&self, num_points: i32) -> Vec<Point2D>;
        fn trajectory_3d(&self, num_points: i32) -> Vec<Point3D>;
        fn angle_rad(&self, other: &dyn Locatable, time: f64) -> f64 {
            let (x, y) = self.xy(time);
            let (x2, y2) = other.xy(time);
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

        fn orbital_period(&self) -> f64 {
            match self.parent.orbital_period() {
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
            (radius * true_anom.cos(), radius * true_anom.sin())
        }
        
        fn in_parent_coordinates(&self, orbit_loc: Point2D) -> Point3D {
            let (ox, oy) = orbit_loc;
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
            (x, z, y)
        }
    }

    pub enum SolarSystemObject<'a> {
        Static { name: &'a str, color: &'a RGBColor, s: StaticObject },
        Moving { name: &'a str, color: &'a RGBColor, o: Orbitor<'a> },
    }

    impl<'a> SolarSystemObject<'a> {
        pub fn new_static(name: &'a str, color: &'a RGBColor, mass: f64, x: f64, y: f64, z: f64) -> SolarSystemObject<'a> {
            SolarSystemObject::Static {
                name: name,
                color: color,
                s: StaticObject::new(mass, x, y, z)
            }
        }

        pub fn new_orbitor(name: &'a str,
                           color: &'a RGBColor,
                           mass: f64,
                           parent: &'a SolarSystemObject<'a>,
                           semimajor: f64,
                           eccentricity: f64,
                           inclination: f64,
                           lan: f64,
                           aop: f64,
                           mae: f64) -> SolarSystemObject<'a> {
            SolarSystemObject::Moving {
                name: name,
                color: color,
                o: Orbitor::new(mass, parent, semimajor, eccentricity,
                    deg_to_rad(inclination), deg_to_rad(lan), deg_to_rad(aop), deg_to_rad(mae))}
        }

        pub fn get_name(&self) -> &str {
            match self {
                Self::Static { name, .. } => name,
                Self::Moving { name, .. } => name,
            }
        }

        pub fn get_color(&self) -> &RGBColor {
            match self {
                Self::Static { color, .. } => color,
                Self::Moving { color, .. } => color,
            }
        }

        pub fn get_mass(&self) -> f64 {
            match self {
                Self::Static { s, .. } => s.mass,
                Self::Moving { o, .. } => o.mass,
            }
        }

        pub fn orbital_period(&self) -> Option<f64> {
            match self {
                Self::Moving { o, .. } => Some (o.orbital_period()),
                Self::Static { .. } => None,
            }
        }
    }

    pub struct SolarSystem<'a> {
        objects: Vec<&'a SolarSystemObject<'a>>
    }

    impl<'a> SolarSystem<'a> {
        pub fn new() -> SolarSystem<'a> {
            SolarSystem {
                objects: Vec::new(),
            }
        }

        pub fn add(&mut self, obj: &'a SolarSystemObject<'a>) {
            self.objects.push(obj)
        }

        pub fn objects(&'a self) -> &'a Vec<&'a SolarSystemObject<'a>> {
            &(self.objects)
        }
    }

    impl Locatable for Orbitor<'_> {
        fn xyz(&self, time: f64) -> Point3D {
            let (x, y, z) = self.parent.xyz(time);
            let (x2, y2, z2) = self.in_parent_coordinates(self.orbit_xy(time));
            ((x + x2 / SCALING_FACTOR), (y + y2 / SCALING_FACTOR), (z + z2 / SCALING_FACTOR))
        }

        fn xy(&self, time: f64) -> Point2D {
            let (x, _, z) = self.xyz(time);
            // println!("{x} {y}");
            (x, z)
        }

        fn trajectory_2d(&self, num_points: i32) -> Vec<Point2D> {
            let mut output = Vec::new();
            for time in (0..num_points+1).map(|x| x as f64 * self.orbital_period() / num_points as f64) {
                let (x, y) = self.xy(time);
                output.push((x, y));
            }
            output
        }

        fn trajectory_3d(&self, num_points: i32) -> Vec<Point3D> {
            let mut output = Vec::new();
            for time in (0..num_points+1).map(|x| x as f64 * self.orbital_period() / num_points as f64) {
                let (x, y, z) = self.xyz(time);
                output.push((x, y, z));
            }
            output
        }
    }

    impl Locatable for StaticObject {
        fn xyz(&self, _time: f64) -> Point3D {
            (self.x, self.y, self.z)
        }

        fn xy(&self, _time: f64) -> Point2D {
            (self.x, self.z)
        }

        fn trajectory_2d(&self, _num_points: i32) -> Vec<Point2D> {
            let mut output = Vec::new();
            // for time in (0..=7).map(|x| x as f64 * TAU / 7.0) {
            //     output.push((self.x + (time - 0.2).cos() / 3.0, self.z + (time - 0.2).cos() / 3.0));
            //     output.push((self.x + time.cos(), self.z + time.sin()));
            // }
            output.push((self.x, self.z));
            output
        }

        fn trajectory_3d(&self, _num_points: i32) -> Vec<Point3D> {
            let mut output = Vec::new();
            // for time in (0..=7).map(|x| x as f64 * TAU / 7.0) {
            //     output.push((self.x + (time - 0.2).cos() / 3.0, self.y + (time - 0.2).cos() / 3.0, self.z + (time - 0.2).cos() / 3.0));
            //     output.push((self.x + time.cos(), self.y + time.sin(), self.z + time.sin() * time.cos()));
            // }
            output.push((self.x, self.y, self.z));
            output
        }
    }

    impl Locatable for SolarSystemObject<'_> {
        fn xyz(&self, time: f64) -> Point3D {
            match self {
                Self::Static { s, .. } => s.xyz(time),
                Self::Moving { o, .. } => o.xyz(time),
            }
        }

        fn xy(&self, time: f64) -> Point2D {
            match self {
                Self::Static { s, .. } => s.xy(time),
                Self::Moving { o, .. } => o.xy(time),
            }
        }

        fn trajectory_2d(&self, num_points: i32) -> Vec<Point2D> {
            match self {
                Self::Static { s, .. } => s.trajectory_2d(num_points),
                Self::Moving { o, .. } => o.trajectory_2d(num_points),
            }
        }

        fn trajectory_3d(&self, num_points: i32) -> Vec<Point3D> {
            match self {
                Self::Static { s, .. } => s.trajectory_3d(num_points),
                Self::Moving { o, .. } => o.trajectory_3d(num_points),
            }
        }
    }
}

