use image::{open, ImageBuffer, Rgb, RgbImage};
use rand::random;
use std::env;
use std::time::Instant;

/// Draws a random single colored rectangle on the image at given coordinates.
/// Returns the random color and the pixel values that were overwritten
fn draw_rectangle(
    image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    top_left: (u32, u32),
    bottom_right: (u32, u32),
) -> (Vec<Vec<Rgb<u8>>>, Rgb<u8>) {
    let color = Rgb([random(), random(), random()]);
    let mut original_pixels = Vec::new();
    for x in top_left.0..bottom_right.0 {
        let mut column = Vec::new();
        for y in top_left.1..bottom_right.1 {
            column.push(*image.get_pixel(x, y));
            image.put_pixel(x, y, color);
        }
        original_pixels.push(column);
    }
    (original_pixels, color)
}

/// Draw a rectangle on the given image at a random location with a random color.
/// Returns the color and the top-left and bottom-right vertices of the rectangle
fn get_neighbor(
    image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
) -> ((Vec<Vec<Rgb<u8>>>, Rgb<u8>), ((u32, u32), (u32, u32))) {
    let (w, h) = image.dimensions();
    let bottom_right = (random::<u32>() % (w + 1), random::<u32>() % (h + 1));
    // if `bottom_right` contains any 0s, we must account for that
    // because we can't perform `n % 0`
    let top_left = match bottom_right {
        (0, 0) => (0, 0),
        (0, y2) => (0, random::<u32>() % y2),
        (x2, 0) => (random::<u32>() % x2, 0),
        _ => (
            random::<u32>() % bottom_right.0,
            random::<u32>() % bottom_right.1,
        ),
    };
    let (original_pixels, new_color) = draw_rectangle(image, top_left, bottom_right);
    ((original_pixels, new_color), (top_left, bottom_right))
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
/// subtracts the pixel differences between the original image and the generated image for a
/// given area, adds back in the pixel differences between the original image and the new color
/// and then calculates the new distance result
fn update_cost(
    previous_cost: f64,
    original_image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    old_pixels: &Vec<Vec<Rgb<u8>>>,
    new_color: Rgb<u8>,
    top_left: (u32, u32),
    bottom_right: (u32, u32),
) -> f64 {
    let (w, h) = original_image.dimensions();
    // restoring the sum from `get_cost`
    let mut s = (previous_cost * previous_cost * (w * h * 3) as f64)
        .sqrt()
        .round() as u64;
    // subtracting off the relevant pixels from the first generated image.
    // also storing `original_image`'s pixels so we don't have to fetch them again
    // because apparently `get_pixel` is an expensive operation??
    let mut original_pixels = Vec::new();
    for x in top_left.0..bottom_right.0 {
        let mut column = Vec::new();
        for y in top_left.1..bottom_right.1 {
            let original_pixel = *original_image.get_pixel(x, y);
            let old_pixel = old_pixels[(x - top_left.0) as usize][(y - top_left.1) as usize];
            s -= pixel_difference(original_pixel, old_pixel);
            column.push(original_pixel);
        }
        original_pixels.push(column);
    }
    // adding in the relevant pixels from the second generated image
    for x in top_left.0..bottom_right.0 {
        for y in top_left.1..bottom_right.1 {
            s += pixel_difference(
                original_pixels[(x - top_left.0) as usize][(y - top_left.1) as usize],
                new_color,
            );
        }
    }
    // recalculating the distance
    let dist = ((s as f64 * s as f64) / ((w * h * 3) as f64)).sqrt();
    dist
}

/// Simulated annealing algorithm to approximate a given image
fn anneal(
    original_image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    alpha: f64,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let initial_temp = 1e3;
    let final_temp = 0.001;
    let mut current_temp = initial_temp;
    let mut image = RgbImage::new(original_image.dimensions().0, original_image.dimensions().1);
    let mut cost = get_cost(&original_image, &image);

    let total_time_start = Instant::now();
    while current_temp >= final_temp {
        let ((image_pixels, new_color), (top_left, bottom_right)) = get_neighbor(&mut image);
        let neighbor_cost = update_cost(
            cost,
            original_image,
            &image_pixels,
            new_color,
            top_left,
            bottom_right,
        );
        let cost_diff = neighbor_cost - cost;
        if cost_diff < 0.0 {
            cost = neighbor_cost;
        } else if random::<f64>() < (-cost_diff / current_temp).exp() {
            cost = neighbor_cost;
        } else {
            // reset pixels to the old image pixels instead of neighbor pixels
            for x in top_left.0..bottom_right.0 {
                for y in top_left.1..bottom_right.1 {
                    image.put_pixel(
                        x,
                        y,
                        image_pixels[(x - top_left.0) as usize][(y - top_left.1) as usize],
                    );
                }
            }
        }
        current_temp *= alpha;
        print!(
            "temperature: {current_temp}\r",
        );
    }

    let total_time_elapsed = total_time_start.elapsed();
    println!(
        "\ntotal time elapsed: {} seconds",
        total_time_elapsed.as_secs_f64()
    );
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
