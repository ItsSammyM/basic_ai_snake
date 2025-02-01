use std::fmt::Display;
use rand::{
    rngs::StdRng,
    seq::IteratorRandom,
    SeedableRng
};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    #[default]
    Empty,
    Head,
    Tail{life: u8},
    Fruit,
}
//10 by 10
const GRID_SIZE: (i32, i32) = (10, 10);
pub struct Grid {
    cells: [CellState; GRID_SIZE.0 as usize * GRID_SIZE.1 as usize],
}
impl Grid {
    fn new() -> Self {
        let mut cells = [CellState::Empty; GRID_SIZE.0 as usize * GRID_SIZE.1 as usize];
        cells[45] = CellState::Head;
        Self {cells}
    }
    fn get(&self, point: &GridPoint) -> Option<&CellState> {
        self.cells.get(point.to_index() as usize)
    }
    fn get_mut(&mut self, point: &GridPoint) -> Option<&mut CellState> {
        self.cells.get_mut(point.to_index() as usize)
    }
    fn set(&mut self, point: &GridPoint, state: CellState) {
        if let Some(cell) = self.cells.get_mut(point.to_index() as usize) {
            *cell = state;
        }
    }
    fn first(&self, cell: CellState) -> Option<GridPoint> {
        self.all(cell).first().cloned()
    }
    fn all(&self, cell: CellState) -> Vec<GridPoint> {
        self.cells
            .iter()
            .enumerate()
            .filter(|(_,c)| **c == cell)
            .map(|(i, _)| GridPoint::from_index(i as i32))
            .flatten()
            .collect()
    }
}
impl Display for Grid{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, cell) in self.cells.iter().enumerate() {
            if i % GRID_SIZE.0 as usize == 0 {
                writeln!(f)?;
            }
            write!(f, "{}", match cell {
                CellState::Empty => ". ",
                CellState::Head => "H ",
                CellState::Tail{..} => "T ",
                CellState::Fruit => "F ",
            })?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
pub struct GridPoint(i32);
impl GridPoint {
    fn from_point(x: i32, y: i32) -> Option<Self> {
        if x < GRID_SIZE.0 && y < GRID_SIZE.1 && x >= 0 && y >= 0 {
            Some(Self(x + y * GRID_SIZE.0))
        } else {
            None
        }
    }
    fn from_index(index: i32) -> Option<Self> {
        if index < GRID_SIZE.0 * GRID_SIZE.1 && index >= 0 {
            Some(Self(index))
        } else {
            None
        }
    }
    fn to_index(&self) -> i32 {
        self.0
    }
    fn to_point(&self) -> (i32, i32) {
        (self.0 % GRID_SIZE.0, self.0 / GRID_SIZE.0)
    }
    fn add(&self, direction: &Direction) -> Option<Self> {
        let (x1, y1) = self.to_point();
        let (x2, y2) = direction.to_point();
        Self::from_point(x1 + x2, y1 + y2)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
impl Direction {
    pub fn to_point(&self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}
pub enum SnakeGame{
    Game(Game),
    GameOver{score: u8},
}
impl SnakeGame{
    pub fn new(seed: u64) -> Self {
        let rng: StdRng = StdRng::seed_from_u64(seed);
        Self::Game(Game::new(rng))
    }
    pub fn accept_input(&mut self, input: Direction) {
        match self {
            Self::Game(game) => game.accept_input(input),
            Self::GameOver{..} => {},
        }
    }
    pub fn to_next_frame(&mut self) -> EndFrameState {
        match self {
            Self::Game(game) => {
                let next_frame_out = game.to_next_frame();
                if let EndFrameState::GameOver{score} = next_frame_out {
                    *self = Self::GameOver{score};
                }
                next_frame_out
            },
            Self::GameOver{score} => EndFrameState::GameOver{score: *score},
        }
    }
    pub fn print_frame(&self) {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("{}", self);
    }
    pub fn neighboring_cell(&self, direction: Direction) -> CellState {
        let SnakeGame::Game(game) = self else {return CellState::Tail { life: 0 }};
        let head = game.grid.first(CellState::Head).expect("head should exist");

        let Some(neighboring_cell) = head.add(&direction) else {
            return CellState::Tail { life: 0 };
        };

        game.grid.get(&neighboring_cell).copied().unwrap_or(CellState::Tail { life: 0 })
    }
    pub fn length(&self) -> u8 {
        let SnakeGame::Game(game) = self else {return 0};
        game.length
    }
    pub fn current_direction(&self) -> Direction {
        let SnakeGame::Game(game) = self else {return Direction::Right};
        game.current_direction
    }

    fn food_direction(&self) -> Vec<Direction> {
        let SnakeGame::Game(game) = self else {return Vec::new()};
        let Some(food_pos) = game.grid.first(CellState::Fruit) else {return Vec::new()};
        let Some(head_pos) = game.grid.first(CellState::Head) else {return Vec::new()};

        let mut directions = Vec::new();
        let (hx, hy) = head_pos.to_point();
        let (fx, fy) = food_pos.to_point();
        if hx < fx {
            directions.push(Direction::Right);
        } else if hx > fx {
            directions.push(Direction::Left);
        }
        if hy < fy {
            directions.push(Direction::Down);
        } else if hy > fy {
            directions.push(Direction::Up);
        }

        directions
    }
    

    pub fn obstacle_direction_up(&self) -> bool {
        matches!(
            self.neighboring_cell(Direction::Up),
            CellState::Tail {..}
        )
    }
    pub fn obstacle_direction_down(&self) -> bool {
        matches!(
            self.neighboring_cell(Direction::Down),
            CellState::Tail {..}
        )
    }
    pub fn obstacle_direction_right(&self) -> bool {
        matches!(
            self.neighboring_cell(Direction::Right),
            CellState::Tail {..}
        )
    }
    pub fn obstacle_direction_left(&self) -> bool {
        matches!(
            self.neighboring_cell(Direction::Left),
            CellState::Tail {..}
        )
    }

    pub fn current_direction_up(&self) -> bool {
        self.current_direction() == Direction::Up
    }
    pub fn current_direction_down(&self) -> bool {
        self.current_direction() == Direction::Down
    }
    pub fn current_direction_right(&self) -> bool {
        self.current_direction() == Direction::Right
    }
    pub fn current_direction_left(&self) -> bool {
        self.current_direction() == Direction::Left
    }

    pub fn food_direction_up(&self) -> bool {
        self.food_direction().contains(&Direction::Up)
    }
    pub fn food_direction_down(&self) -> bool {
        self.food_direction().contains(&Direction::Down)
    }
    pub fn food_direction_right(&self) -> bool {
        self.food_direction().contains(&Direction::Right)
    }
    pub fn food_direction_left(&self) -> bool {
        self.food_direction().contains(&Direction::Left)
    }
}
impl Display for SnakeGame{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Game(game) => write!(f, "{}", game),
            Self::GameOver{score} => write!(f, "Game Over, Score: {}", score),
        }
    }
}
#[derive(PartialEq, Eq)]
pub enum EndFrameState{
    Continue,
    GameOver{score: u8},
}
pub struct Game {
    grid: Grid,
    current_direction: Direction,
    length: u8, //kill all tails with life >= length
    rng: StdRng,
}
impl Game {
    fn new(rng: StdRng) -> Self {
        Self {
            grid: Grid::new(),
            current_direction: Direction::Right,
            length: 1,
            rng
        }
    }
    fn accept_input(&mut self, input: Direction) {
        self.current_direction = input;
    }
    fn to_next_frame(&mut self)->EndFrameState{
        self.kill_tails();
        self.increase_life();
        let move_head_out = self.move_head();
        if matches!(move_head_out, EndFrameState::GameOver{..}) {
            return move_head_out;
        }

        if self.count_fruits() == 0 {
            self.spawn_fruit(1);
        }
        EndFrameState::Continue
    }
    fn move_head(&mut self)->EndFrameState{
        let Some(head_pos) = self.grid.first(CellState::Head) else {return EndFrameState::GameOver{score: self.length}};
        let Some(new_head_pos) = head_pos.add(&self.current_direction) else {return EndFrameState::GameOver{score: self.length}};
        
        match self.grid.get(&new_head_pos) {
            Some(CellState::Empty) => {
                self.grid.set(&head_pos, CellState::Tail{life: 0});
                self.grid.set(&new_head_pos, CellState::Head);
                EndFrameState::Continue
            },
            Some(CellState::Fruit) => {
                self.grid.set(&head_pos, CellState::Tail{life: 0});
                self.grid.set(&new_head_pos, CellState::Head);
                self.length += 1;
                EndFrameState::Continue
            },
            _ => EndFrameState::GameOver{score: self.length},
        }
    }
    fn increase_life(&mut self) {
        for cell in self.grid.cells.iter_mut() {
            if let CellState::Tail{life} = cell {
                *life += 1;
            }
        }
    }
    fn kill_tails(&mut self) {
        for cell in self.grid.cells.iter_mut() {
            if let CellState::Tail{life} = cell {
                if *life >= self.length {
                    *cell = CellState::Empty;
                }
            }
        }
    }
    fn count_fruits(&self) -> u8 {
        self.grid.cells.iter().filter(|c| **c == CellState::Fruit).count() as u8
    }
    fn spawn_fruit(&mut self, amount: u8) {
        let all_empty_cells: Vec<GridPoint> = self.grid.cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| **cell == CellState::Empty)
            .map(|(pos, _)| GridPoint::from_index(pos as i32)
            .expect("index should be in bounds because were enumerating over the cells"))
            .choose_multiple(&mut self.rng, amount as usize);

        for cell in all_empty_cells {
            if let Some(cell) = self.grid.get_mut(&cell) {
                *cell = CellState::Fruit;
            }
        }
    }
}
impl Display for Game{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.grid)?;
        write!(f, "\n{:?}", self.current_direction)
    }
}


