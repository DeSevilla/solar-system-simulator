pub mod orbitor {
    use std::f64::consts::TAU;

    use plotters::style::RGBColor;

    type Point2D = (f64, f64);
    type Point3D = (f64, f64, f64);

    pub trait Locatable {
        fn xyz(&self, time: f64) -> Point3D;
        fn xy(&self, time: f64) -> Point2D;
    }

    pub struct StaticObject {
        x: f64,
        y: f64,
        z: f64
    }
    
    impl StaticObject {
        pub fn new(x: f64, y: f64, z:f64) -> StaticObject {
            StaticObject {
                x: x,
                y: y,
                z: z,
            }
        }
    }

    pub struct Orbitor<'a> {
        parent: &'a SolarSystemObject<'a>,
        semimajor: f64,
        eccentricity: f64,
        inclination: f64,
        lan: f64, //longitude of the ascending node
        aop: f64, //argument of periapsis
        mae: f64, //mean anomaly at epoch
    }

    pub enum SolarSystemObject<'a> {
        Static { name: &'a str, color: &'a RGBColor, s: StaticObject },
        Moving { name: &'a str, color: &'a RGBColor, o: Orbitor<'a> },
    }

    impl<'a> SolarSystemObject<'a> {
        pub fn new_static(name: &'a str, color: &'a RGBColor, x: f64, y: f64, z: f64) -> SolarSystemObject<'a> {
            SolarSystemObject::Static {
                name: name,
                color: color,
                s: StaticObject::new(x, y, z)
            }
        }

        pub fn new_orbitor(name: &'a str,
                           color: &'a RGBColor,
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
                o: Orbitor::new(parent, semimajor, eccentricity, inclination, lan, aop, mae)}
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

    impl<'a> Orbitor<'a> {
        pub fn new(parent: &'a SolarSystemObject<'a>, 
                   semimajor: f64, 
                   eccentricity: f64, 
                   inclination: f64, 
                   lan: f64,
                   aop: f64,
                   mae: f64) -> Orbitor<'a> {
            Orbitor {
                parent: parent,
                semimajor: semimajor,
                eccentricity: eccentricity,
                inclination: inclination,
                lan: lan,
                aop: aop,
                mae: mae,
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
                (self.mae + time * (1.32712440041e+20/self.semimajor.powi(3)).sqrt()) % TAU
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
            (x, y, z)
        }
    }

    impl Locatable for Orbitor<'_> {
        fn xyz(&self, time: f64) -> Point3D {
            let (x, y, z) = self.parent.xyz(time);
            let (x2, y2, z2) = self.in_parent_coordinates(self.orbit_xy(time));
            (x + x2, y + y2, z + z2)
        }

        fn xy(&self, time: f64) -> Point2D {
            let (x, y, _) = self.xyz(time);
            // println!("{x} {y}");
            (x / 1000000.0, y / 1000000.0)
        }
    }

    impl Locatable for StaticObject {
        fn xyz(&self, _time: f64) -> Point3D {
            (self.x, self.y, self.z)
        }

        fn xy(&self, time: f64) -> Point2D {
            (self.x + time.cos(), self.y + time.sin())
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
    }
}

