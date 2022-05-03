fn main_old() {
    let r = Point {
        x: 123.0,
        y: 123.0,
    };

    if r.x > 0.0 && r.y > 0.0 {
        println!("Point at {} is in the first quadrant!", r.to_string());
    }

    let p = r.to_polar();

    println!("The polar equivalent is {}!", p.to_string());
}

struct Point {
    x: f64,
    y: f64,
}

struct Polar {
    r: f64,
    deg: f64,
}

impl Point {
    fn to_string(&self) -> String {
        let res = format!("({}, {})", self.x.to_string(), self.y.to_string());
        res
    }
}

impl Point {
    fn to_polar(&self) -> Polar {
        Polar { r: (self.x.powi(2) + self.y.powi(2)).sqrt(), deg: (self.y / self.x).tan() }
    }
}

impl Polar {
    fn to_string(&self) -> String {
        let res =  format!("({}, {}rad)", self.r.to_string(), self.deg.to_string());
        res
    }
}