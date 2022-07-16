mod utils;

use std::{collections::HashMap, iter::FromIterator, str::from_utf8};

use itertools::Itertools;
use ndarray::{s, stack, Array1, Array2, Array3, ArrayView1, ArrayView2, ArrayView3, Axis};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen(getter_with_clone)]
pub struct BoardoptResult {
    pub score: f64,
    pub placed: String,
    pub boosted: String,
}

const GETROLLS: [f64; 256] = {
    let mut getrolls = [0.0; 256];
    getrolls[b'@' as usize] = 1.0;
    getrolls[b'*' as usize] = 2.0;
    getrolls[b'N' as usize] = 0.2;
    getrolls[b'C' as usize] = 0.1;
    getrolls
};

fn matrix_high_power(a: ArrayView2<f64>) -> Array2<f64> {
    let mut tmp = a.to_owned();
    for _ in 0..6 {
        tmp = tmp.dot(&tmp);
    }
    tmp
}

fn get_dice_pow(n: usize) -> Array3<f64> {
    let mut dice = Array2::<f64>::zeros((n, n));
    for ((i, j), x) in dice.indexed_iter_mut() {
        let val = (j as i32 - i as i32).rem_euclid(n as i32);
        if val > 0 && val <= 6 {
            *x = 1.0 / 6.0;
        }
    }
    // println!("{}", &dice);
    let mut dice_pow = Array3::<f64>::zeros((6, n, n));
    let mut tmp = Array2::<f64>::eye(n);
    for i in 0..6 {
        dice_pow.slice_mut(s![i, .., ..]).assign(&tmp);
        tmp = tmp.dot(&dice);
    }
    dice_pow
}

fn get_steady(tiles: &[i32], dice_pow: ArrayView3<f64>) -> (Array1<f64>, Array2<f64>) {
    let extra: Array2<f64> = stack(
        Axis(0),
        &tiles
            .iter()
            .copied()
            .enumerate()
            .map(|(i, tile)| dice_pow.slice(s![tile, i, ..]))
            .collect_vec(),
    )
    .unwrap();
    let dice_pow_2: ArrayView2<f64> = dice_pow.slice(s![2, .., ..]);
    let trans = dice_pow_2.dot(&matrix_high_power(extra.view()));
    let steady = matrix_high_power(trans.view()).slice(s![0, ..]).to_owned();
    (steady, trans)
}

fn board_to_tiles(board: &[u8]) -> Vec<i32> {
    board
        .iter()
        .copied()
        .map(|x| {
            if x.is_ascii_digit() {
                (x - b'0') as i32
            } else {
                0
            }
        })
        .collect_vec()
}

fn objective(
    board: &[u8],
    steady: ArrayView1<f64>,
    trans: ArrayView2<f64>,
    weights: &[f64; 256],
    booster: i32,
) -> (f64, Vec<bool>) {
    let w = Array1::from_vec(
        board
            .iter()
            .copied()
            .map(|x| weights[x as usize])
            .collect_vec(),
    );
    let getroll = Array1::from_vec(
        board
            .iter()
            .copied()
            .map(|x| GETROLLS[x as usize])
            .collect_vec(),
    );
    let nroll = 1.0 - getroll;
    let score = trans.dot(&w);
    let roll = trans.dot(&nroll);

    let mut sum_s = steady.dot(&score);
    let mut sum_r = steady.dot(&roll);
    let mut boosts = vec![false; board.len()];
    for (i, (s, r)) in score
        .iter()
        .copied()
        .zip(roll.iter().copied())
        .enumerate()
        .sorted_by(|&(_, (s1, r1)), &(_, (s2, r2))| (s2 / r2).total_cmp(&(s1 / r1)))
    {
        if s / r <= sum_s / sum_r {
            break;
        }
        boosts[i] = true;
        sum_s += (booster - 1) as f64 * steady[i] * s;
        sum_r += (booster - 1) as f64 * steady[i] * r;
    }
    ((sum_s / sum_r), boosts)
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, boardopt-wasm!");
}

#[wasm_bindgen]
pub fn boardopt(
    boards: &JsValue,
    lms1: &str,
    lms2: &str,
    min_num_lm: i32,
    max_num_lm: i32,
    weights: &JsValue,
    booster: i32,
) -> BoardoptResult {
    utils::set_panic_hook();

    let boards: Vec<String> = boards.into_serde().unwrap();
    let boards = boards.iter().map(|x| x.as_bytes()).collect_vec();
    let lms1 = lms1.as_bytes();
    let lms2 = lms2.as_bytes();
    let weights: HashMap<String, f64> = weights.into_serde().unwrap();
    let weights = {
        let mut w = [0.0; 256];
        for (k, v) in weights {
            w[k.as_bytes()[0] as usize] = v;
        }
        w
    };

    let mut best_score = f64::NEG_INFINITY;
    let mut best_placed = Vec::new();
    let mut best_boosted = Vec::new();

    for &board in &boards {
        let (submin, submax) = if from_utf8(board).unwrap().contains("55") {
            (min_num_lm - 1, max_num_lm - 1)
        } else {
            (min_num_lm, max_num_lm)
        };
        let min1 = std::cmp::max(0, submin - lms2.len() as i32);
        let max1 = std::cmp::min(lms1.len() as i32, submax - lms2.len() as i32);
        let lm_list1 = (min1..=max1)
            .flat_map(|i| lms1.iter().copied().permutations(i as usize).unique())
            .collect_vec();

        let dice_pow = get_dice_pow(board.len());
        let slots = board
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, x)| x == b'#')
            .map(|(i, _)| i)
            .collect_vec();
        for landmarks in &lm_list1 {
            let result = slots
                .iter()
                .copied()
                .combinations(landmarks.len())
                // .par_bridge()
                .map(|indices| {
                    let mut best_score = f64::NEG_INFINITY;
                    let mut best_placed = Vec::new();
                    let mut best_boosted = Vec::new();
                    // for indices in slots.iter().copied().combinations(landmarks.len()) {
                    let mut placed = Vec::from(board);
                    for (i, landmark) in indices.iter().copied().zip(landmarks.iter().copied()) {
                        placed[i] = landmark;
                    }
                    let slots2 = placed
                        .iter()
                        .copied()
                        .enumerate()
                        .filter(|&(_, x)| x == b'#')
                        .map(|(i, _)| i)
                        .collect_vec();

                    let lm_list2 = lms2
                        .iter()
                        .copied()
                        .permutations(lms2.len())
                        .unique()
                        .collect_vec();

                    let tiles = board_to_tiles(&placed);
                    let (steady, trans) = get_steady(&tiles, dice_pow.view());

                    for landmarks2 in &lm_list2 {
                        for indices2 in slots2.iter().copied().combinations(landmarks2.len()) {
                            let mut placed2 = placed.clone();
                            for (i, landmark) in
                                indices2.iter().copied().zip(landmarks2.iter().copied())
                            {
                                placed2[i] = landmark;
                            }
                            let (score, boosted) =
                                objective(&placed2, steady.view(), trans.view(), &weights, booster);
                            if score > best_score {
                                best_score = score;
                                best_placed = placed2.clone();
                                best_boosted = boosted.clone();
                            }
                        }
                    }
                    (best_score, best_placed, best_boosted)
                })
                .max_by(|&(a, _, _), &(b, _, _)| a.total_cmp(&b));
            if let Some((score, placed, boosted)) = result {
                if score > best_score {
                    best_score = score;
                    best_placed = placed;
                    best_boosted = boosted;
                }
            }
        }
        // println!("{}  {}", from_utf8(&best_placed).unwrap(), best_score);
        // println!(
        //     "{}",
        //     from_utf8(
        //         &best_boosted
        //             .iter()
        //             .copied()
        //             .map(|x| if x { b'^' } else { b' ' })
        //             .collect_vec()
        //     )
        //     .unwrap()
        // );
    }

    BoardoptResult {
        score: best_score,
        placed: String::from_utf8(best_placed).unwrap(),
        boosted: String::from_iter(best_boosted.into_iter().map(|x| if x { '^' } else { ' ' })),
    }
}
