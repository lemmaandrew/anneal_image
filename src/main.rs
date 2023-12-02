use clap::Parser;
use image::{open, ImageBuffer, Rgb, RgbImage};
use rand::random;
use rayon::prelude::*;
use std::{iter::zip, time::Instant};

/// Gets the coordinates of a random single-colored triangle with the given vertices.
/// Returns said coordinates and the random color that it should be filled with
/// Algorithm stolen from http://www.sunshine2k.de/coding/java/TriangleRasterization/TriangleRasterization.html
fn get_triangle(vertices: &mut [(u32, u32); 3]) -> (Vec<(u32, u32)>, Rgb<u8>) {
    fn flat_bottom_triangle([v1, v2, v3]: &[(i64, i64); 3]) -> Vec<(u32, u32)> {
        let invslope1 = (v2.0 - v1.0) as f64 / (v2.1 - v1.1) as f64;
        let invslope2 = (v3.0 - v1.0) as f64 / (v3.1 - v1.1) as f64;
        let mut curx1 = v1.0 as f64;
        let mut curx2 = v1.0 as f64;
        let mut coords = Vec::new();
        for y in v1.1..=v2.1 {
            coords.extend((curx1 as u32..=curx2 as u32).map(|x| (x, y as u32)));
            curx1 += invslope1;
            curx2 += invslope2;
        }

        coords
    }

    fn flat_top_triangle([v1, v2, v3]: &[(i64, i64); 3]) -> Vec<(u32, u32)> {
        let invslope1 = (v3.0 - v1.0) as f64 / (v3.1 - v1.1) as f64;
        let invslope2 = (v3.0 - v2.0) as f64 / (v3.1 - v2.1) as f64;
        let mut curx1 = v3.0 as f64;
        let mut curx2 = v3.0 as f64;
        let mut coords = Vec::new();
        for y in (v1.1 + 1..=v3.1).rev() {
            coords.extend((curx1 as u32..=curx2 as u32).map(|x| (x, y as u32)));
            curx1 -= invslope1;
            curx2 -= invslope2;
        }

        coords
    }

    vertices.sort_by_key(|v| (v.1, v.0));
    let vt1 = (vertices[0].0 as i64, vertices[0].1 as i64);
    let vt2 = (vertices[1].0 as i64, vertices[1].1 as i64);
    let vt3 = (vertices[2].0 as i64, vertices[2].1 as i64);
    let color: Rgb<u8> = Rgb([random(), random(), random()]);

    if vt2.1 == vt3.1 {
        (flat_bottom_triangle(&[vt1, vt2, vt3]), color)
    } else if vt1.1 == vt2.1 {
        (flat_top_triangle(&[vt1, vt2, vt3]), color)
    } else {
        // splitting triangle into top half and bottom half
        let mut coords = Vec::new();
        let x4 = (vt1.0 as f64
            + ((vt2.1 - vt1.1) as f64 / (vt3.1 - vt1.1) as f64) * (vt3.0 - vt1.0) as f64)
            as i64;
        let vt4 = (x4, vt2.1);
        let mut flat_bottom = [vt1, vt2, vt4];
        flat_bottom.sort_by_key(|v| (v.1, v.0));
        let mut flat_top = [vt2, vt4, vt3];
        flat_top.sort_by_key(|v| (v.1, v.0));
        coords.extend(flat_bottom_triangle(&flat_bottom));
        coords.extend(flat_top_triangle(&flat_top));
        (coords, color)
    }
}

/// Draws a random single-colored rectangle on the image at given coordinates.
/// Returns the pixels that were modified (coordinates and original colors) and the random color.
fn get_rectangle(top_left: (u32, u32), bottom_right: (u32, u32)) -> (Vec<(u32, u32)>, Rgb<u8>) {
    let color = Rgb([random(), random(), random()]);
    let mut coords = Vec::new();
    for x in top_left.0..bottom_right.0 {
        for y in top_left.1..bottom_right.1 {
            coords.push((x, y));
        }
    }
    (coords, color)
}

/// Draw a rectangle on the given image at a random location with a random color.
/// Returns the color and the top-left and bottom-right vertices of the rectangle
fn get_neighbor(
    image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    triangle: bool,
) -> (Vec<(u32, u32)>, Rgb<u8>) {
    let (w, h) = image.dimensions();
    if !triangle {
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
        get_rectangle(top_left, bottom_right)
    } else {
        let v1 = (random::<u32>() % w, random::<u32>() % h);
        let v2 = (random::<u32>() % w, random::<u32>() % h);
        let v3 = (random::<u32>() % w, random::<u32>() % h);
        // ensuring we have a valid triangle
        if v1 == v2
            || v2 == v3
            || v1 == v3
            || v1.0 == v2.0 && v2.0 == v3.0
            || v1.1 == v2.1 && v2.1 == v3.1
        {
            get_neighbor(image, triangle)
        } else {
            get_triangle(&mut [v1, v2, v3])
        }
    }
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
    annealed_image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    coords: &Vec<(u32, u32)>,
    new_color: Rgb<u8>,
    sample: Option<u32>,
) -> f64 {
    let (w, h) = original_image.dimensions();
    // restoring the sum from `get_cost`
    let mut s = (previous_cost * previous_cost * (w * h * 3) as f64).sqrt();
    match sample {
        None => {
            // storing `original_image`'s pixels so we don't have to fetch them again
            // because apparently `get_pixel` is an expensive operation??
            let original_pixels = coords
                .par_iter()
                .map(|(x, y)| *original_image.get_pixel(*x, *y))
                .collect::<Vec<Rgb<u8>>>();
            let annealed_pixels = coords
                .par_iter()
                .map(|(x, y)| *annealed_image.get_pixel(*x, *y))
                .collect::<Vec<Rgb<u8>>>();
            // subtracting off the relevant pixels from the first generated image.
            s -= (0..original_pixels.len())
                .into_par_iter()
                .map(|i| pixel_difference(original_pixels[i], annealed_pixels[i]) as f64)
                .sum::<f64>();
            // adding in the relevant pixels from the second generated image
            s += original_pixels
                .par_iter()
                .map(|pixel| pixel_difference(*pixel, new_color) as f64)
                .sum::<f64>();
        }
        Some(n) => {
            // getting a linspace of indices to sample from
            let dx = (coords.len() - 1) as f64 / (n - 1) as f64;
            let sample_indices = (0..n)
                .map(|i| (i as f64 * dx) as usize)
                .collect::<Vec<usize>>();
            // sampling the original image
            let original_pixels_sample = if (n as usize) < coords.len() {
                sample_indices
                    .iter()
                    .map(|&i| {
                        let (x, y) = coords[i];
                        *original_image.get_pixel(x, y)
                    })
                    .collect::<Vec<Rgb<u8>>>()
            } else {
                coords
                    .iter()
                    .map(|(x, y)| *original_image.get_pixel(*x, *y))
                    .collect::<Vec<Rgb<u8>>>()
            };
            // sampling the old pixels
            let annealed_sample = if (n as usize) < coords.len() {
                sample_indices
                    .iter()
                    .map(|&i| {
                        let (x, y) = coords[i];
                        *annealed_image.get_pixel(x, y)
                    })
                    .collect::<Vec<Rgb<u8>>>()
            } else {
                coords
                    .iter()
                    .map(|(x, y)| *annealed_image.get_pixel(*x, *y))
                    .collect::<Vec<Rgb<u8>>>()
            };
            // subtracting off the pixel differences between the original image and the old pixels
            s -= zip(original_pixels_sample.clone(), annealed_sample)
                .map(|(pixel1, pixel2)| pixel_difference(pixel1, pixel2) as f64)
                .sum::<f64>();
            // adding back in the pixel differences between the original image andda the new color
            s += original_pixels_sample
                .iter()
                .map(|&pixel| pixel_difference(pixel, new_color) as f64)
                .sum::<f64>();
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
    triangle: bool,
    sample: Option<u32>,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let initial_temp = 1e3;
    let final_temp = 0.001;
    let mut current_temp = initial_temp;
    let mut image = RgbImage::new(original_image.dimensions().0, original_image.dimensions().1);
    let mut cost = get_cost(&original_image, &image);

    let total_time_start = Instant::now();
    while current_temp >= final_temp {
        let (coords, new_color) = get_neighbor(&mut image, triangle);
        let neighbor_cost = update_cost(cost, original_image, &image, &coords, new_color, sample);
        let cost_diff = neighbor_cost - cost;
        if cost_diff < 0.0 || random::<f64>() < (-cost_diff / current_temp).exp() {
            cost = neighbor_cost;
            for (x, y) in coords.iter() {
                image.put_pixel(*x, *y, new_color);
            }
        }
        current_temp *= alpha;
        print!("temperature: {current_temp}\r",);
    }

    let total_time_elapsed = total_time_start.elapsed();
    println!(
        "\ntotal time elapsed: {} seconds",
        total_time_elapsed.as_secs_f64()
    );
    image
}

#[derive(Parser)]
struct Args {
    /// Input image path
    #[arg(short, long)]
    input: String,

    /// Output image path
    #[arg(short, long)]
    output: String,

    /// Temperature change value
    #[arg(short, long, default_value_t = 0.999)]
    alpha: f64,

    /// Flag for drawing triangles instead of rectangles
    #[arg(short, long)]
    triangle: bool,

    /// Randomly sample pixels for cost calculation.
    /// Much faster than non-sampled, at the cost of loss of accuracy
    #[arg(short, long)]
    sample: Option<u32>,
}

fn main() {
    let args = Args::parse();
    let original_image = open(args.input).unwrap().into_rgb8();
    let generated_image = anneal(&original_image, args.alpha, args.triangle, args.sample);
    generated_image.save(args.output).unwrap();
}
