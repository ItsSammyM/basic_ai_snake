use std::{env, fs::File, io::Write, ops::Div};
use matrix::ColVector;
use network::Network;
use rand::Rng;
use serde::{Deserialize, Serialize};
use snake_game::{Direction, EndFrameState, SnakeGame};

mod snake_game;
mod network;
mod matrix;

const FILE_NAME: &str = "generation(12,12,4).json";
const NUM_NETS: usize = 100;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 32 megabytes of stack space
    std::thread::Builder::new().stack_size(32 * 1024 * 1024).spawn(move || {
        if let Some(string) = args.get(1){
            match string.as_str() {
                "train" => {
                    train_networks(FILE_NAME);
                },
                "test" => {
                    Generation::load(FILE_NAME).show_best_ever_network();
                },
                "play" => {
                    let mut game = SnakeGame::new(0);
                    loop {
                        game.accept_input(get_input_from_console());
                        if matches!(game.to_next_frame(), EndFrameState::GameOver{..}) {
                            break;
                        }
                        game.print_frame();
                    }
                },
                _ => {
                    println!("Invalid argument");
                    return;
                }
            }
        }

    }).unwrap().join().unwrap();
}

type Score = f32;

#[derive(Clone, Serialize, Deserialize)]
struct Generation{
    best_ever_network: (Network, Score),
    networks: Vec<(Network, Score)>,
    generation_counter: u64,
}
impl Generation{
    fn new(num_networks: usize)->Self{
        Self{
            best_ever_network: (Network::new(&mut rand::thread_rng()), 0.0),
            networks: (0..num_networks).map(|_| (Network::new(&mut rand::thread_rng()), 0.0)).collect(),
            generation_counter: 0,
        }
    }
    fn new_from_generation(parents: &Generation)->Self{
        let rand = &mut rand::thread_rng();

        let (_, max) = parents.score_range(1.0f32);

        let num_parents = parents.networks.len();
        let mut new_networks: Vec<(Network, Score)> = parents.networks
            .iter()
            .filter_map(|(network, score)|{

                let percent_score = score.div(max);

                if rand.gen::<f32>() % 1.0 < percent_score * percent_score {
                    Some((network.clone(), *score))
                }else{
                    None
                }

            })
            .collect();

        new_networks.push(parents.best_ever_network.clone());

        let num_culled_networks = new_networks.len();

        let mut i = 0;
        while new_networks.len() < num_parents {

            if let Some(parent) = new_networks.get(i % num_culled_networks) {

                let mut new_network = parent.0.clone();
                new_network.randomly_edit(rand);
                new_networks.push((new_network, 0.0));
                
            }else{
                i = 0;
            }

            i += 1;
        }

        
        let mut out = Self{
            best_ever_network: parents.best_ever_network.clone(),
            networks: new_networks,
            generation_counter: parents.generation_counter.saturating_add(1),
        };
        
        out.train_scores();

        out
    }
    fn train_scores(&mut self){
        self.networks = train_scores_on_multiple_threads(self.networks.drain(..).collect(), 6);
        
        self.networks.sort_by(|a, b|
            b.1.partial_cmp(&a.1).unwrap()
        );
        
        if let Some(first) = self.networks.first(){
            if first.1 > self.best_ever_network.1 {
                self.best_ever_network = first.clone();
            }
        }
    }
    fn show_best_ever_network(&self){
        let mut game = SnakeGame::new(self.generation_counter);

        loop {
            let direction = get_input_from_network(&game, &self.best_ever_network.0);
            game.accept_input(direction);
            
            
            if matches!(game.to_next_frame(), EndFrameState::GameOver{..}) {
                break;
            }

            game.print_frame();
            println!("input: {:?}", direction);
    
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    fn mean_score(&self, top_percent: f32)->f32{
        let num = (self.networks.len() as f32 * top_percent) as usize;
        self.networks
            .iter()
            .take(num)
            .map(|(_, score)| *score as f32)
            .sum::<f32>() / num as f32
    }
    
    /// (Min, Max)
    fn score_range(&self, top_percent: f32)->(Score, Score){
        let num = (self.networks.len() as f32 * top_percent) as usize;
        self.networks
            .iter()
            .take(num)
            .map(|(_, score)| *score as f32)
            .fold((std::f32::MAX, std::f32::MIN), |(min, max), score| {
                (min.min(score), max.max(score))
            })
    }

    fn save(&self, path: &str){
        let Ok(string) = serde_json::to_string(self) else {
            println!("failed to serialize");
            return;
        };

        let Ok(mut file) = File::create(path) else {
            println!("failed to create file");
            return;
        };

        let Ok(_) = file.write_all(string.as_bytes()) else {
            println!("failed to write to file");
            return;
        };
        
        println!("Saved generation");
    }
    fn load(path: &str)->Self{
        let Ok(string) = std::fs::read_to_string(path) else {
            println!("failed to read file, generating random");
            return Self::new(NUM_NETS);
        };

        let Ok(generation) = serde_json::from_str::<Generation>(&string) else {
            println!("failed to deserialize, generating random");
            return Self::new(NUM_NETS);
        };

        println!("loaded generation {}", path);

        generation
    }
}


fn train_scores_on_multiple_threads(mut networks: Vec<(Network, Score)>, num_threads: u8)->Vec<(Network, Score)>{
    for i in 0..num_threads {
        let start_index = (networks.len() / num_threads as usize) * (num_threads - i) as usize;

        let split_vec: Vec<(Network, f32)> = networks.split_off(start_index);

        let join_handle = std::thread::spawn(move || {
            return train_scores_single_thread(split_vec);
        });

        networks.append(&mut join_handle.join().unwrap());
    }
    
    networks
}
fn train_scores_single_thread(mut networks: Vec<(Network, Score)>)->Vec<(Network, Score)>{
    for (network, score) in networks.iter_mut(){
        *score = get_score(network, 0);
    }
    networks
}




fn train_networks(file_name: &str) {

    let mut generation = Generation::load(file_name);

    loop {
        generation = Generation::new_from_generation(&generation);
        if generation.generation_counter % 100 == 0 {
            println!(
                "Generation {}: Best Ever: {}, Avg score: {}",
                generation.generation_counter,
                generation.best_ever_network.1,
                generation.mean_score(0.3f32),
            );
            generation.save(file_name);
        }
    }
    
}

fn get_score(net: &Network, seed: u64) -> f32 {

    const NUM_SIMULATIONS: i32 = 3;

    let mut avg_score = 0;

    for i in 0..NUM_SIMULATIONS {
        
        let mut game = SnakeGame::new(seed+(i as u64));
        let mut num_frames: i32 = 0;
        
        //run game
        let score = loop {
            game.accept_input(get_input_from_network(&game, &net));

            num_frames += 1;
            
            if let EndFrameState::GameOver{score} = game.to_next_frame() {
                break score as i32;
            }

            let snake_length = game.length() as i32;
            if num_frames > (200 + (snake_length * 50)) {
                break snake_length;
            }

        };

        avg_score += score;
    }

    avg_score as f32 / NUM_SIMULATIONS as f32
}


fn get_input_from_console() -> Direction {
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "w" => return Direction::Up,
            "s" => return Direction::Down,
            "a" => return Direction::Left,
            "d" => return Direction::Right,
            _ => println!("Invalid input"),
        }
    }
}

fn get_input_from_network(game: &SnakeGame, net: &Network) -> Direction {
    let input = 
        ColVector::new_from_slice([
            [if game.obstacle_direction_up() { 1.0 } else { 0.0 };1],
            [if game.obstacle_direction_down() { 1.0 } else { 0.0 };1],
            [if game.obstacle_direction_left() { 1.0 } else { 0.0 };1],
            [if game.obstacle_direction_right() { 1.0 } else { 0.0 };1],
            [if game.current_direction_up() { 1.0 } else { 0.0 };1],
            [if game.current_direction_down() { 1.0 } else { 0.0 };1],
            [if game.current_direction_right() { 1.0 } else { 0.0 };1],
            [if game.current_direction_left() { 1.0 } else { 0.0 };1],
            [if game.food_direction_up() { 1.0 } else { 0.0 };1],
            [if game.food_direction_down() { 1.0 } else { 0.0 };1],
            [if game.food_direction_right() { 1.0 } else { 0.0 };1],
            [if game.food_direction_left() { 1.0 } else { 0.0 };1],
        ]);

    let out = match net.choice_with_highest_confidence(input) {
        0 => Direction::Up,
        1 => Direction::Down,
        2 => Direction::Left,
        3 => Direction::Right,
        _ => panic!("Invalid output from network"),
    };
    out
}