use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, self};
use rand::prelude::*;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
struct Board {
    board: [[Space; 8]; 8],
    row_walls: [i32; 8],
    col_walls: [i32; 8]
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // print columns
        write!(f, "  ")?;
        for i in 0..8 {
            write!(f, "{} ", self.col_walls[i])?;
        }
        writeln!(f, "")?;
        for row in 0..8 {
            write!(f, "{} ", self.row_walls[row])?;
            for col in 0..8 {
                write!(f, "{} ", self.board[row][col].to_str())?;
            }
            writeln!(f, "")?;
        }

        Ok(())
    }
}

impl Index<(usize, usize)> for Board {
    type Output = Space;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        
        if index.0 >= 8 || index.1 >= 8 {
            return &Space::Wall;
        }
        &self.board[index.0 as usize][index.1 as usize]
    }
}

impl IndexMut<(usize, usize)> for Board {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.board[index.0 as usize][index.1 as usize]
    }
}

impl Board {
    pub fn new() -> Self {
        use Space::*;
        Board {
            board: [[Empty; 8]; 8],
            row_walls: [0; 8],
            col_walls: [0; 8],
        }
    }

    pub fn get(&self, r: i32, c: i32) -> Space {
        if r >= 8 || r < 0 || c >= 8 || c < 0 {
            return Space::Wall;
        }
        self[(r as usize, c as usize)]
    }

    pub fn initialize(&mut self) {
        let mut rng = rand::thread_rng();
        for r in 0..8 {
            for c in 0..8 {
                let random: bool = rng.gen();
                if self.board[r][c].can_change() {
                    if random {
                        self[(r, c)] = Space::Wall;
                    } else {
                        self[(r, c)] = Space::Empty;
                    }
                }
            }
        }
    }

    // make a random valid change to the board and return a new board
    pub fn step(&self) -> Self {
        let mut new_board = self.clone();

        let mut rng = rand::thread_rng();
        let mut r = rng.gen();
        r = r % 8;
        let mut c = rng.gen();
        c = c % 8;

        while !self[(r, c)].can_change() {
            r = rng.gen();
            r = r % 8;

            c = rng.gen();
            c = c % 8;
        }

        if self[(r, c)] == Space::Wall {
            new_board[(r, c)] = Space::Empty;
        } else {
            new_board[(r, c)] = Space::Wall;
        }

        new_board
    }

    pub fn cost(&self) -> i32 {
        let mut cost = 0;
        let b = self.board;
        let mut chests = Vec::with_capacity(5);

        // row and column wall counts
        {
            let mut row_vals = [0; 8];
            let mut col_vals = [0; 8];

            for r in 0..8 {
                for c in 0..8 {
                    if b[r][c] == Space::Wall {
                        row_vals[r] += 1;
                        col_vals[c] += 1;
                    }

                    if b[r][c] == Space::Chest {
                        chests.push((r, c));
                    }
                }
            }

            for i in 0..8 {
                cost += (row_vals[i] - self.row_walls[i]).abs();
                cost += (col_vals[i] - self.col_walls[i]).abs();
            }
        }

        // chests
        let mut board_mask = [[false; 8]; 8];
        {
            // locate chests
            for (r, c) in chests {
                let (r, c) = (r as i32, c as i32);
                let mut best_centre = (0, 0);
                let mut best_score = 0;
                for i in -1..2 {
                    if i + r > 6 || i + r < 1 { continue; }
                    for j in -1..2 {
                        if j + c > 6 || j + c < 1 { continue; }
                        let mut score = 0;
                        for r_offset in -1..2 {
                            for c_offset in -1..2 {
                                if self[((r+i+r_offset) as usize, (c+j+c_offset) as usize)] != Space::Wall {
                                    score += 1;
                                }
                            }
                        }


                        if score > best_score {
                            best_centre = (r+i, c+j);
                            best_score = score;
                        }
                    }
                }

                
                let mut entrance_count = 0;
                
                // check perimiter of chest room
                for i in -1..2 {
                    if self.get(best_centre.0 - 2, best_centre.1 + i) != Space::Wall {
                        entrance_count += 1;
                    }
                    if self.get(best_centre.0 + 2, best_centre.1 + i) != Space::Wall {
                        entrance_count += 1;
                    }
                    if self.get(best_centre.0 + i, best_centre.1 - 2) != Space::Wall {
                        entrance_count += 1;
                    }
                    if self.get(best_centre.0 + i, best_centre.1 + 2) != Space::Wall {
                        entrance_count += 1;
                    }
                }

                cost += 9 - best_score;

                if entrance_count == 0 {
                    cost += 10;
                } else {
                    cost += entrance_count - 1;
                }

                // mask in array
                for i in -1..2 {
                    for j in -1..2 {
                        board_mask[(best_centre.0+i) as usize][(best_centre.1+j) as usize] = true;
                    }
                }
            }
        }

        // check for 2x2s
        {
            for i in 0..7 {
                for j in 0..7 {
                    let mut empty_2x2 = true;
                    let mut is_masked = true;
                    for i_off in 0..2 {
                        for j_off in 0..2 {
                            // check if all masked

                            if !board_mask[i+i_off][j+j_off] {
                                is_masked = false;
                            }
                            if self[(i+i_off, j+j_off)] == Space::Wall {
                                empty_2x2 = false;
                            }
                        }
                    }
                    if empty_2x2 && !is_masked {
                        cost += 5;
                    }
                }
            }
        }


        // count disconnected regions
        {
            let mut dfs_filler: Vec<(i32, i32)> = Vec::with_capacity(32);
            let mut board_mask = [[false; 8]; 8];
            let mut regions = 0;

            // find any empty non-masked space
            loop {
                'outer_loop:
                for i in 0..8 {
                    for j in 0..8 {
                        if !board_mask[i][j] && self[(i, j)] != Space::Wall {
                            dfs_filler.push((i as i32, j as i32));
                            board_mask[i][j] = true;
                            regions += 1;
                            break 'outer_loop;
                        }
                    }
                }

                if dfs_filler.len() == 0 { break; }

                while dfs_filler.len() > 0 {
                    let current_pos = dfs_filler.pop().unwrap();
                    let offsets = [(0, 1), (1, 0), (-1, 0), (0, -1)];
                    
                    for o in offsets {
                        let r = current_pos.0 as i32 + o.0;
                        let c = current_pos.1 as i32 + o.1;
                        if self.get(r, c) != Space::Wall && !board_mask[r as usize][c as usize] {
                            dfs_filler.push((r, c));
                            board_mask[r as usize][c as usize] = true;
                        }
                    }
                }
            }

            cost += (regions - 1) * 20;
        }

        // dead end checks
        {
            // on each empty space, must have <3 neighboring walls or monster
            for i in 0..8 {
                for j in 0..8 {
                    if self[(i as usize, j as usize)] == Space::Wall { continue; }
                    let offsets = [(0, 1), (1, 0), (-1, 0), (0, -1)];
                    let mut count = 0;
                    for o in offsets {
                        let r = i as i32 + o.0;
                        let c = j as i32 + o.1;
                        if self.get(r, c) == Space::Wall {
                            count += 1;
                        }
                    }

                    if count == 3 && self[(i as usize, j as usize)] != Space::Monster {
                        cost += 4;
                    }
                    if self[(i as usize, j as usize)] == Space::Monster && count != 3 {
                        cost += 4;
                    }
                }
            }
        }


        cost
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Space {
    Empty,
    Chest,
    Monster,
    Wall,
    Unassigned
}

impl Space {
    fn to_str(&self) -> &'static str {
        use Space::*;
        match self {
            Empty => " ",
            Chest => "0",
            Monster => "!",
            Wall => "#",
            Unassigned => " "
        }
    }

    fn can_change(&self) -> bool {
        use Space::*;
        match self {
            Empty => true,
            Wall => true,
            Unassigned => true,
            _ => false,
        }
    }
}

fn main() {
    // read input
    let file = File::open("input.txt").unwrap();
    let line_iter = io::BufReader::new(file).lines();
    let mut rng = rand::thread_rng();

    let mut lines: Vec<String> = Vec::new();
    for line in line_iter {
        if let Ok(l) = line {
            lines.push(l);
        }
    }

    let column_walls: Vec<i32> = lines[0].to_string().trim().split(' ').map(|e| str::parse::<i32>(e).unwrap()).collect();

    let mut row_walls = Vec::new();
    let mut board: Board = Board::new();

    for (idx, line) in (&lines[1..]).iter().enumerate() {
        let walls = str::parse::<i32>(&line[0..1]).unwrap();
        row_walls.push(walls);

        let board_row: Vec<Space> = line[1..].to_string().trim().split(' ')
            .map(|e| match e {
                "#" => Space::Empty,
                "C" => Space::Chest,
                "M" => Space::Monster,
                "W" => Space::Wall,
                _ => Space::Unassigned
            }).collect();

        for i in 0..8 {
            board.board[idx][i] = board_row[i];
        }
    }

    for i in 0..8 {
        board.row_walls[i] = row_walls[i];
        board.col_walls[i] = column_walls[i];
    }

    board.initialize();
    // let mut temperature = 0;

    let mut cost = board.cost();

    let mut c = 0;
    let mut resets = 0;
    let max_iters = 40000;

    while cost > 0 {
        let new_board = board.step();
        let new_cost = new_board.cost();

        let temperature = 1.0 - ((c as f64) / max_iters as f64);

        if new_cost < cost || ((-(new_cost - cost)) as f64/temperature).exp() > rng.gen_range(0.0..1.0) {
            board = new_board;
            cost = new_cost;
        }

        c += 1;

        if c % 1000 == 0 {
            // print!("{}", board);
            // println!("Cost: {}, Temperature: {}", cost, temperature);
        }

        if c >= max_iters {
            c = 0;
            board.initialize();
            cost = board.cost();
            resets += 1;
        }
    }
    print!("{}", board);
    println!("solved in {} iterations and {} resets", c, resets);
    
}
