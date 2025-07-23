fn main() {
    println!("Hello, world!");
}

fn square(a: f64) -> f64 {
    a * a
}

fn radians_from_degrees(degrees: f64) -> f64 {
    0.01745329251994329577 * degrees
}

fn haversine(x0: f64, y0: f64, x1: f64, y1: f64, earth_radius: f64) -> f64 {
    let lat1 = y0;
    let lat2 = y1;
    let lon1 = x0;
    let lon2 = x1;

    let d_lat = radians_from_degrees(lat2 - lat1);
    let d_lon = radians_from_degrees(lon2 - lon1);
    let lat1 = radians_from_degrees(lat1);
    let lat2 = radians_from_degrees(lat2);

    let a = square(f64::sin(d_lat / 2.0))
        + f64::cos(lat1) * f64::cos(lat2) * square(f64::sin(d_lon / 2.0));
    let c = 2.0 * f64::asin(f64::sqrt(a));

    earth_radius * c
}
