use image::{ImageError, Rgb, RgbImage};
use rand::prelude::*;
use rand_distr::Normal;

type Color = [f64; 3];

fn subdivide_noise<R: Rng>(img: &Vec<Vec<Color>>, noise: f64, rng: &mut R) -> Vec<Vec<Color>> {
    assert_eq!(img.len(), img[0].len());
    let new_size = img.len() * 2;
    let mut new_image: Vec<Vec<Color>> = vec![vec![[0.0; 3]; new_size]; new_size];
    let normal = Normal::new(0.0, noise).unwrap();
    for (r, row) in img.iter().enumerate() {
        for (c, color) in row.iter().enumerate() {
            for (dr, dc) in vec![(0, 0), (0, 1), (1, 0), (1, 1)] {
                let new_color =
                    color.map(|channel| (channel + normal.sample(rng)).clamp(0.0, 255.0));
                new_image[r * 2 + dr][c * 2 + dc] = new_color;
            }
        }
    }
    new_image
}

type Pixel = (usize, usize);
fn neighbors(pixel: Pixel, size: usize) -> Vec<Pixel> {
    let (r, c) = pixel;
    vec![(-1, 0), (0, -1), (1, 0), (0, 1)]
        .into_iter()
        .map(|(dr, dc)| {
            (
                (r as isize + dr + size as isize) as usize % size,
                (c as isize + dc + size as isize) as usize % size,
            )
        })
        .collect()
}

fn score(img: &Vec<Vec<Color>>, center: Pixel, outerp: f64, group: &Vec<Pixel>) -> f64 {
    let (r, c) = center;
    let color = img[r][c];
    group
        .iter()
        .map(|&(gr, gc)| {
            let group_color = img[gr][gc];
            (0..3)
                .map(|i| (color[i] - group_color[i]).powi(2))
                .sum::<f64>()
                .powf(outerp)
        })
        .sum()
}
fn perform_exchanges<R: Rng>(
    img: &mut Vec<Vec<Color>>,
    exchanges_per_pixel: usize,
    outerp: f64,
    rng: &mut R,
) {
    let size = img.len();
    let num_exchanges = size.pow(2) * exchanges_per_pixel;
    for _ in 0..num_exchanges {
        let p1 = (rng.gen_range(0..img.len()), rng.gen_range(0..img.len()));
        let p2 = (rng.gen_range(0..img.len()), rng.gen_range(0..img.len()));
        let neighbors1 = neighbors(p1, size);
        let neighbors2 = neighbors(p2, size);
        let self_score =
            score(&img, p1, outerp, &neighbors1) + score(&img, p2, outerp, &neighbors2);
        let swap_score =
            score(&img, p1, outerp, &neighbors2) + score(&img, p2, outerp, &neighbors1);
        if swap_score < self_score {
            let col1 = img[p1.0][p1.1];
            let col2 = img[p2.0][p2.1];
            img[p1.0][p1.1] = col2;
            img[p2.0][p2.1] = col1;
        }
    }
}
fn make_image(
    initial_noise: f64,
    final_noise: f64,
    num_subdivide: usize,
    outerp: f64,
    exchanges_per_pixel_max: usize,
    seed: u64,
) -> RgbImage {
    assert!(initial_noise >= final_noise);
    let mut img: Vec<Vec<Color>> = vec![vec![[128.0; 3]]];
    let mut rng = StdRng::seed_from_u64(seed);
    let noise_step = (final_noise / initial_noise).powf(1.0 / ((num_subdivide - 1) as f64));
    for i_subdivide in 0..num_subdivide {
        let noise = initial_noise * noise_step.powi(i_subdivide as i32);
        img = subdivide_noise(&img, noise, &mut rng);
        let exchanges_per_pixel =
            exchanges_per_pixel_max * 2usize.pow((num_subdivide - 1 - i_subdivide) as u32);
        perform_exchanges(&mut img, exchanges_per_pixel, outerp, &mut rng);
    }
    let mut out_img = RgbImage::new(img.len() as u32, img.len() as u32);
    for (r, row) in img.iter().enumerate() {
        for (c, color) in row.iter().enumerate() {
            let round_color: [u8; 3] = color.map(|f| f.round() as u8);
            out_img.put_pixel(r as u32, c as u32, Rgb(round_color));
        }
    }
    out_img
}

fn main() -> Result<(), ImageError> {
    let initial_noise = 255;
    let final_noise = 4;
    let outerp = 0.1;
    let exchanges_per_pixel_max = 1000;
    let seed = 0;
    let num_subdivide = 10;
    let filename = format!(
        "img-{}-{}-{}-{}-{}-{}.png",
        initial_noise, final_noise, num_subdivide, outerp, exchanges_per_pixel_max, seed
    );
    println!("{}", filename);
    let img = make_image(
        initial_noise as f64,
        final_noise as f64,
        num_subdivide,
        outerp,
        exchanges_per_pixel_max,
        seed,
    );
    img.save(filename)
}
