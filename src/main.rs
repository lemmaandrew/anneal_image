use image::{open, ImageBuffer, Rgb, RgbImage};
use rand::random;
use std::env;

/// Draws a random single colored rectangle on the image at given coordinates.
/// Returns the slice of the original rectangle
fn draw_rectangle(
    image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    top_left: (u32, u32),
    bottom_right: (u32, u32),
) {
    let color = Rgb([random(), random(), random()]);
    for y in top_left.1..bottom_right.1 {
        for x in top_left.0..bottom_right.0 {
            image.put_pixel(x, y, color);
        }
    }
}

/// Draw a rectangle on the given image at a random location with a random color
fn get_neighbor(
    mut image: ImageBuffer<Rgb<u8>, Vec<u8>>,
) -> (ImageBuffer<Rgb<u8>, Vec<u8>>, ((u32, u32), (u32, u32))) {
    let (w, h) = image.dimensions();
    let bottom_right = (random::<u32>() % (w + 1), random::<u32>() % (h + 1));
    let top_left = match bottom_right {
        (0, 0) => (0, 0),
        (0, y2) => (0, random::<u32>() % y2),
        (x2, 0) => (random::<u32>() % x2, 0),
        _ => (
            random::<u32>() % bottom_right.0,
            random::<u32>() % bottom_right.1,
        ),
    };
    draw_rectangle(&mut image, top_left, bottom_right);
    (image, (top_left, bottom_right))
}

/// Difference between two pixels as a single value
fn pixel_difference(pixel1: Rgb<u8>, pixel2: Rgb<u8>) -> u64 {
    let [r1, g1, b1] = pixel1.0;
    let [r2, g2, b2] = pixel2.0;
    ((r1 as i32 - r2 as i32).abs() + (g1 as i32 - g2 as i32).abs() + (b1 as i32 - b2 as i32).abs())
        as u64
}

/// RMSE difference between the original image and the generated image
fn get_cost(
    original_image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    generated_image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
) -> f64 {
    let (w, h) = original_image.dimensions();
    let mut s = 0;
    for x in 0..w {
        for y in 0..h {
            let &pixel1 = original_image.get_pixel(x, y);
            let &pixel2 = generated_image.get_pixel(x, y);
            s += pixel_difference(pixel1, pixel2);
        }
    }

    let dist = ((s as f64 * s as f64) / ((w * h * 3) as f64)).sqrt();
    dist
}

/// A less expensive version of `get_cost`.
/// Takes a previous `get_cost` result, resets it to the sum of pixel differences,
/// subtracts the pixel differences between the original image and the first generated image for a
/// given area, adds back in the pixel differences between the original image and the second
/// generated image for that area, then returns the total distance calculation.
fn update_cost(
    previous_cost: f64,
    original_image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    generated_image1: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    generated_image2: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    top_left: (u32, u32),
    bottom_right: (u32, u32),
) -> f64 {
    let (w, h) = original_image.dimensions();
    let mut s = (previous_cost * previous_cost * (w * h * 3) as f64)
        .sqrt()
        .round() as u64;
    for x in top_left.0..bottom_right.0 {
        for y in top_left.1..bottom_right.1 {
            s -= pixel_difference(
                *original_image.get_pixel(x, y),
                *generated_image1.get_pixel(x, y),
            );
        }
    }
    for x in top_left.0..bottom_right.0 {
        for y in top_left.1..bottom_right.1 {
            s += pixel_difference(
                *original_image.get_pixel(x, y),
                *generated_image2.get_pixel(x, y),
            );
        }
    }
    let dist = ((s as f64 * s as f64) / ((w * h * 3) as f64)).sqrt();
    dist
}

fn anneal(
    original_image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    alpha: f64,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let initial_temp = 1e3;
    let final_temp = 0.001;
    let mut current_temp = initial_temp;
    let mut image = RgbImage::new(original_image.dimensions().0, original_image.dimensions().1);
    let mut cost = get_cost(&original_image, &image);

    while current_temp >= final_temp {
        print!("{current_temp}\r");
        let (neighbor, (top_left, bottom_right)) = get_neighbor(image.clone());
        let neighbor_cost = update_cost(
            cost,
            original_image,
            &image,
            &neighbor,
            top_left,
            bottom_right,
        );
        let cost_diff = neighbor_cost - cost;
        if cost_diff < 0.0 {
            image = neighbor;
            cost = neighbor_cost;
        } else if random::<f64>() < (-cost_diff / current_temp).exp() {
            image = neighbor;
            cost = neighbor_cost;
        }
        current_temp *= alpha;
    }

    image
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let path = args[1].clone();
    let dest = args[2].clone();
    let alpha = match args.get(3) {
        Some(a) => a.parse().unwrap(),
        None => 0.999,
    };
    let original_image = open(path).unwrap().into_rgb8();
    let generated_image = anneal(&original_image, alpha);
    generated_image.save(dest).unwrap();
}
