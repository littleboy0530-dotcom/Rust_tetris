///기본 인덱스
/// V방향 -> Y+, ->방향 X+
/// 모든 인덱스는 y , x 순서로 적는다

//필요한 라이브러리, 함수 import
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{Clear, ClearType,enable_raw_mode},
    style::Print,
};
use std::{
    io::{stdout, Write},
    thread,
    time::{Duration, Instant},
};
use rand::{seq::SliceRandom, Rng};
use rand::thread_rng;

//게임 유틸리티 함수

/// ms 단위로 sleep하는 함수
#[inline]
fn sleep_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

/// 논블로킹 입력 함수
/// 입력 없으면 '0' 반환
/// 입력된 키가 소문자 a-z면 그대로 반환, 아니면 '0'
fn input() -> char {
    if event::poll(Duration::from_millis(10)).unwrap_or(false) {
        if let Ok(Event::Key(key_event)) = event::read() {
            // 키가 눌린 상태(Press)일 때만 처리
            if key_event.kind == KeyEventKind::Press {
                if let KeyCode::Char(c) = key_event.code {
                    if c.is_ascii_lowercase() {
                        return c;
                    }
                }
            }
        }
    }
    '0'
}

/// 화면 지우기 함수 (플리커링 최소화 적용)
fn clear() {
    execute!(stdout(), MoveTo(0, 0)).unwrap();
}

fn clear_one() {
    let mut stdout = stdout();

    execute!(
        stdout,
        Print("\x1b[3J"),       // 스크롤백 포함 전체 지우기
        Clear(ClearType::All),  // 전체 화면 지우기
        MoveTo(0, 0)            // 커서를 좌상단으로 이동
    ).unwrap();

    stdout.flush().unwrap();
}

const ORIGIN_LOC:(i32,i32) = (1 , 4);
const WALLKICK_OFFSET: [(i32, i32);5] = [
    (0,0),
    (0,-1),
    (0,1),
    (-1,0),
    (1,0)
];

#[derive(Clone, Copy, PartialEq)]
enum Mino {
    O,
    I,
    Z,
    S,
    L,
    T,
    J,
    None,
}

impl Mino {
    fn _random() -> Self {
        let mut rng = thread_rng();
        let minos: Vec<Mino> = vec![Mino::O, Mino::T, Mino::I, Mino::L, Mino::J, Mino::S, Mino::Z];
        let r = rng.gen_range(0..7);
        minos[r]
    }

    fn shape(&self) -> [[char;4];4] {
        let result = match self {
            Mino::I => {
                [[' ',' ',' ',' '],
                 ['#','#','#','#'],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            },
            Mino::J => {
                [['#',' ',' ',' '],
                 ['#','#','#',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            },
            Mino::L => {
                [[' ',' ','#',' '],
                 ['#','#','#',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            },
            Mino::None => {
                [[' ',' ',' ',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            },
            Mino::O => {
                [['#','#',' ',' '],
                 ['#','#',' ',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            },
            Mino::S => {
                [[' ','#','#',' '],
                 ['#','#',' ',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            },
            Mino::T => {
                [['#','#','#',' '],
                 [' ','#',' ',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            },
            Mino::Z => {
                [['#','#',' ',' '],
                 [' ','#','#',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            }
        };

        result
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Block {
    shape: [[char;4];4],
    origin: (i32, i32),
}

impl Block {
    fn new(kind: Mino) -> Self {
        let shape = match kind {
            Mino::I => {
                [[' ',' ',' ',' '],
                 ['#','#','#','#'],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            },
            Mino::J => {
                [['#',' ',' ',' '],
                 ['#','#','#',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            },
            Mino::L => {
                [[' ',' ','#',' '],
                 ['#','#','#',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            },
            Mino::None => {
                [[' ',' ',' ',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            },
            Mino::O => {
                [['#','#',' ',' '],
                 ['#','#',' ',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            },
            Mino::S => {
                [[' ','#','#',' '],
                 ['#','#',' ',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            },
            Mino::T => {
                [['#','#','#',' '],
                 [' ','#',' ',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            },
            Mino::Z => {
                [['#','#',' ',' '],
                 [' ','#','#',' '],
                 [' ',' ',' ',' '],
                 [' ',' ',' ',' ']]
            }
        };
        
        Block{shape: shape, origin: (1,1)}
    }

    fn to_vec(&self) -> Vec<(i32, i32)> {
        let mut result = Vec::new();
        for (y, line) in self.shape.iter().enumerate() {
            for (x, ch) in line.iter().enumerate() {
                if *ch == '#' {
                    let y = y as i32;
                    let x = x as i32;

                    result.push((y - self.origin.0, x - self.origin.1));
                }

            }
        }

        result
    }

    fn rotate_right(&mut self) {
        let mut rotated = [[' ';4];4];

        for y in 0..4 {
            for x in 0..4 {
                rotated[x][3-y] = self.shape[y][x];
            }
        }

        self.shape = rotated;
        match self.origin {
            (y, x) => {
                self.origin.0 = x;
                self.origin.1 = 3 - y;
            }
        }
    }

    fn rotate_left(&mut self) {
        let mut rotated = [[' '; 4]; 4];

        for y in 0..4 {
            for x in 0..4 {
                rotated[3-x][y] = self.shape[y][x];
            }
        }

        self.shape = rotated;
        match self.origin {
            (y , x) => {
                self.origin.0 = 3 - x;
                self.origin.1 = y;
            }
        }
    }

    fn _debug_print(&self) {
        for line in self.shape {
            for c in line {
                let ch = if c != '#' {
                    '.'
                }else {
                    c
                };

                print!("{}", ch);
            }
            println!();
        }
    }
}
//현재 조종중인 블록
#[derive(Clone, Copy, PartialEq)]
struct Now {
    kind: Mino,
    block: Block,
    loc: (i32, i32),
}

impl Now {
    fn init_with(mino: Mino) -> Self {
        let kind = mino;
        let block = Block::new(mino);
        let loc = ORIGIN_LOC;

        Now { kind: kind, block: block, loc: loc}
    }
}
//다음 블록과 가방
struct Next {
    blocks: [Mino;4],
    bag: Vec<Mino>,
}

impl Next {
    fn new() -> Self {
        let rs_block = [Mino::None;4];
        let rs_bag = Vec::new();
        let mut rs = Next{blocks: rs_block, bag: rs_bag};

        rs.bag_init();
        for i in 0..4 {
            rs.blocks[i] = rs.bag.pop().unwrap();
        }

        rs
    }
    fn bag_init(&mut self) {
        let mut minos: Vec<Mino> = vec![Mino::O, Mino::T, Mino::I, Mino::L, Mino::J, Mino::S, Mino::Z];
        let rng = thread_rng;
        minos.shuffle(&mut rng());

        while !minos.is_empty() {
            self.bag.push(minos.pop().unwrap());
        }
    }

    fn give_block(&mut self) -> Mino {
        let out = self.blocks[0];
        for idx in 0..3 {
            self.blocks[idx] = self.blocks[idx+1];
        }

        if self.bag.is_empty() {
            self.bag_init();
        }

        self.blocks[3] = self.bag.pop().unwrap();

        out
    }
}

//홀드한 블록
struct Hold {
    kind: Mino,
    can_hold: bool,
}

impl Hold {
    fn holding(&mut self, now: &mut Now, next: &mut Next) {
        if !self.can_hold {
            return
        }
        
        if self.kind == Mino::None {
            //홀드한 미노가 없는 상태일 때

            //현재 블록을 홀드에 넣고
            self.kind = now.kind;

            //현재 블록을 다음 블록으로 세팅
            *now = Now::init_with(next.give_block());
        }else {
            //홀드한 미노가 있을 경우

            //서로 블록을 바꿈
            let temp = self.kind;
            self.kind = now.kind;
            *now = Now::init_with(temp);
        }

        self.can_hold = false;
    }

    fn new() -> Self {
        Hold { kind: Mino::None, can_hold: true}
    }
}

struct Board {
    grid: [[char;10];20],
}

impl Board {
    ///보드 위에 현재 블록을 그려서 반환
    fn draw_with(&self, now: &Now) -> [[char;10];20] {
        let mut draw = self.grid;
        let origin = now.loc;
        let block_index_vec: Vec<(i32,i32)>= 
        now.block.to_vec().iter().map(|yx| vec_add(yx, &origin)).collect();

        for idx in block_index_vec {
            if is_in_board(&idx) {
                draw[idx.0 as usize][idx.1 as usize] = '#';
            }
        }

        draw
    }

    ///보드에 현재 블록을 고정
    fn fix_with(&mut self, now: &Now) {
        let origin = now.loc;
        let block_index_vec: Vec<(i32,i32)>= 
        now.block.to_vec().iter().map(|yx| vec_add(yx, &origin)).collect();

        block_index_vec.iter().for_each(|idx| self.grid[idx.0 as usize][idx.1 as usize] = '#');
    }

    ///현재 블록이 해당 위치에 존재 가능한지
    fn is_possible(&self, now: &Now) -> bool {
        let origin = now.loc;
        let block_index_vec: Vec<(i32,i32)>= 
        now.block.to_vec().iter().map(|yx| vec_add(yx, &origin)).collect();

        //현재 블록의 일부가 보드를 벗어 났을 때
        if !block_index_vec.iter().all(|idx| is_in_board(idx)) {
            return false
        }

        //블록의 일부가 겹쳤을 때
        if block_index_vec.iter().all(|idx| self.is_empty(idx)) {
            return true
        }else {
            return false
        }
    }

    ///하드 드롭
    /// 반드시 사용후 현재블록에 다음 블록을 넣을것
    fn hard_drop(&mut self, now: &mut Now) {
        while !is_bottom(&now, self) {
            now.loc.0 += 1;
        }

        self.fix_with(&now);
    }

    ///꽉 찬 줄 지우기
    fn clear_line(&mut self) -> i32 {
        let mut new_grid = [[' ';10];20];
        let mut write_y = 19;
        let mut cleard = 0;

        for cur_y in (0..20).rev() {
            if self.grid[cur_y].iter().all(|ch| *ch == '#') {
                cleard += 1;
            }else {
                new_grid[write_y as usize] = self.grid[cur_y];
                write_y -= 1;
            }
        }
        
        self.grid = new_grid;

        cleard
    }

    ///헬퍼 - 현재 해당위치에 다른 블록이 존재하는가?
    fn is_empty(&self, idx: &(i32,i32)) -> bool {
        let yx = if is_in_board(idx) {
            (idx.0 as usize, idx.1 as usize)
        }else {
            return false
        };

        if self.grid[yx.0][yx.1] == '#' {
            return false
        }else {
            return true
        }
    }

    fn _debug_print(&self) {
        for y in 0..20 {
            for x in 0..10 {
                if self.grid[y][x] == '#' {
                    print!("#");
                }else{
                    print!(".")
                }
            }
            println!();
        }
    }
}

struct GameManager{
    score: u32,
    level: u32,
    tick_rate: Duration,
    cleard: u32,
}

impl GameManager {
    #[inline]
    fn new() -> Self {
        GameManager{score: 0, level: 1, tick_rate: Duration::from_millis(1000), cleard: 0}
    }

    ///자동으로 판정해서 레벨을 올려줌
    ///clear가 0보다 큰 조건 안에 넣을것
    fn level_up(&mut self) {
        if self.cleard / 10 >= self.level {
            self.level += 1;
            let mut ms:u64 = 1100;
            let mut count = self.level;
            while count > 0 {
                let turm = if (1..=6).contains(&count) {
                    100
                }else if (7..=10).contains(&count) {
                    50
                }else if (11..=16).contains(&count) {
                    30
                }else if count <= 19 {
                    20
                }else {
                    10
                };

                ms -= turm;
                count -= 1;
            }

            self.tick_rate = Duration::from_millis(ms);
        }
    }

    fn score_up(&mut self, clear: u32) {
        let lv = self.level;
        let s = match clear {
            1 => 100*lv,
            2 => 300*lv,
            3 => 500*lv,
            4 => 800*lv,
            _ => 0,
        };

        self.score += s;
    }

    fn print_record(&self) {
        println!("최종 레벨: {}", self.level);
        println!("최종 점수: {}", self.score);
        println!("지운 줄의 갯수: {}", self.cleard);
    }
}

///현재 위치를 오른쪽으로
fn move_right(now: &mut Now, board: &Board) {
    let mut trying = *now;
    trying.loc.1 += 1;

    if board.is_possible(&trying) {
        *now = trying;
    }
}

///현재 위치를 왼쪽으로
fn move_left(now: &mut Now, board: &Board) {
    let mut trying = *now;
    trying.loc.1 -= 1;

    if board.is_possible(&trying) {
        *now = trying;
    }
}

fn move_down(now: &mut Now, board: &Board) -> bool {
    if !is_bottom(now, board) {
        now.loc.0 += 1;
        true
    }else{
        false
    }
}

///두 벡터를 참조하여 더해서 반환
#[inline]
fn vec_add(a: &(i32,i32), b:&(i32,i32)) -> (i32, i32) {
    ((a.0 + b.0), (a.1 + b.1))
}

//해당 인덱스가 유효한가
fn is_in_board(case: &(i32,i32)) -> bool {
    if case.0 >= 0 && case.0 < 20 && case.1 >= 0 && case.1 < 10 {
        return true
    }else {
        return false
    }
}

fn is_bottom(now: &Now, board: &Board) -> bool {
    let mut trying = *now;
    trying.loc.0 += 1;

    if !board.is_possible(&trying) {
        return true
    }else{
        return false
    }
}

fn rotate_right_with_kick(now: &mut Now, board: &Board) {
    let mut trying = *now;
    trying.block.rotate_right();

    let offset: [(i32,i32);5] = match now.kind {
        Mino::I =>{
            let rot = get_rot(now);
            let r = get_offset(rot, 1);
            r
        },
        _ => WALLKICK_OFFSET,
    };

    for test in offset {
        let new_loc = vec_add(&test, &now.loc);
        trying.loc = new_loc;

        if board.is_possible(&trying) {
            *now = trying;
            return
        }
    }

}

fn rotate_left_with_kick(now: &mut Now, board: &Board) {
    let mut trying = *now;
    trying.block.rotate_left();

    let offset: [(i32,i32);5] = match now.kind {
        Mino::I =>{
            let rot = get_rot(now);
            let r = get_offset(rot, -1);
            r
        },
        _ => WALLKICK_OFFSET,
    };

    for test in offset {
        let new_loc = vec_add(&test, &now.loc);
        trying.loc = new_loc;

        if board.is_possible(&trying) {
            *now = trying;
            return
        }
    }
}
//헬퍼 - 미노의 회전상태 추적 (0-> 0, R-> 1, 2-> 2, L-> 3)
fn get_rot(now: &Now) -> i32 {
    let ori_yx = now.block.origin;
    let rot = match ori_yx {
        (y, x) => {
            if y == 1 && x == 1 {
                0
            }else if y == 1 && x == 2{
                1
            }else if y == 2 && x == 2{
                2
            }else if y == 2 && x == 1{
                3
            }else {
                panic!("불가능한 원점");
            }
        }
    };

    rot
}

//헬퍼 - 회전 상황에 따라 I미노의 벽킥 오프셋을 반환
fn get_offset(now_rot: i32, direction: i32) -> [(i32,i32);5] {
    let offset = if direction > 0 {
        match now_rot {
            0 => [(0,0), (0,-2), (0,1), (1,-2), (-2,1)],
            1 => [(0,0), (0,-1), (0,2), (2,-1), (-1,2)],
            2 => [(0,0), (0,2), (0,-1), (-1,2), (2,-1)],
            4 => [(0,0), (0,1), (0,-2), (-2,1), (1,-2)],
            _ => [(0,0), (0,0), (0,0), (0,0), (0,0)],
        }
    }else {
        match now_rot {
            0 => [(0,0), (0,-1), (0,2), (2,-1), (-1,2)],
            3 => [(0,0), (0,-2), (0,1), (1,-2), (-2,1)],
            2 => [(0,0), (0,1), (0,-2), (-2,1), (1,-2)],
            1 => [(0,0), (0,2), (0,-1), (-1,2), (2,-1)],
            _ => [(0,0), (0,0), (0,0), (0,0), (0,0)],
        }
    };

    offset
}

fn print_all(board: &[[char;10];20], hold: &Hold, next: &Next, gm: &GameManager) {
    let hold_block = hold.kind.shape();
    let next_block_0 = next.blocks[0].shape();
    let next_block_1 = next.blocks[1].shape();
    let next_block_2 = next.blocks[2].shape();
    let next_block_3 = next.blocks[3].shape();

    for y in 0..22 {
        for x in 0..30 {
            if y == 0 {
                print!("-");
            }else if y == 21 && (10..=19).contains(&x) {
                print!("=");
            }else if (1..=20).contains(&y) && (10..=19).contains(&x) {
                print!("{}", board[y-1][x-10]);
            }else if x == 0 || x == 9 || x == 20 || x == 29 {
                print!("|");
            }else if [6,11,16].contains(&y) && ((1..=8).contains(&x) || (21..=28).contains(&x)) {
                print!("-");
            }else if y == 1 {
                if (3..=6).contains(&x) {
                    if x == 3 {
                        print!("HOLD");
                    }
                }else if (23..=26).contains(&x) {
                    if x == 23 {
                        print!("NEXT");
                    }
                }else {
                    print!(" ");
                }
            }else if (2..=5).contains(&y) && (3..=6).contains(&x) {
                print!("{}",hold_block[y-2][x-3]);
            }else if (2..=5).contains(&y) && (23..=26).contains(&x) {
                print!("{}", next_block_0[y -2][x - 23]);
            }else if y == 7 && (2..=6).contains(&x) {
                if x == 2 {
                    print!("SCORE");
                }
            }else if y == 9 {
                if (1..=8).contains(&x) {
                    if x == 1 {
                        print!("{:08}", gm.score);
                    }
                }else {
                    print!(" ");
                }
            }else if (7..=10).contains(&y) && (23..=26).contains(&x) {
                print!("{}",next_block_1[y-7][x-23]);
            }else if (12..=15).contains(&y) && (23..=26).contains(&x) {
                print!("{}",next_block_2[y-12][x-23]);
            }else if (17..=20).contains(&y) && (23..=26).contains(&x) {
                print!("{}",next_block_3[y-17][x-23]);
            }else if y == 12 && (x == 4 || x == 5) {
                if x == 4 {
                    print!("LV");
                }
            }else if y == 14 && (x == 4 || x == 5) {
                if x == 4 {
                    print!("{:02}",gm.level);
                }
            }else if y == 17 && (3..=6).contains(&x) {
                if x == 3 {
                    print!("LINE");
                }
            }else if y == 19 && (3..=6).contains(&x) {
                if x == 3 {
                    print!("{:04}", gm.cleard);
                }
            }else {
                print!(" ");
            }
        }
        println!();
    }
}
// 012345678901234567890123456789
// ------------------------------  0
// |12HOLD34|0123456789|12NEXT34|  1
// |12####34|0123456789|12####34|  2
// |12####34|0123456789|12####34|  3
// |12####34|0123456789|12####34|  4
// |12####34|0123456789|12####34|  5
// |--------|0123456789|--------|  6
// |12SCORE3|0123456789|12####34|  7
// |12####34|0123456789|12####34|  8
// |(      )|0123456789|12####34|  9
// |12####34|0123456789|12####34|  10
// |--------|0123456789|--------|  1
// |12#LV#34|0123456789|12####34|  2
// |12####34|0123456789|12####34|  3
// |12#()#34|0123456789|12####34|  4
// |12####34|0123456789|12####34|  5
// |--------|0123456789|--------|  6
// |12LINE78|0123456789|12####34|  7
// |12345678|0123456789|12####34|  8
// |12(  )78|0123456789|12####34|  9
// |12345678|==========|12####34|  20
fn main() {
    
    // 시작 시 커서 숨기기
    execute!(stdout(), Hide).unwrap();
    clear_one();

    let mut last_tick = Instant::now();
    let mut lock_timer: Option<Instant> = None;

    let mut next = Next::new();
    let mut now = Now::init_with(next.bag.pop().unwrap());
    let mut hold = Hold::new();
    let mut board = Board{grid: [[' ';10];20]};
    let mut game_manager = GameManager::new();

    loop {
        let ch = input();

        match ch {
            'a' => {move_left(&mut now, &board);},
            'd' => {move_right(&mut now, &board);},
            'k' => {if now.kind != Mino::O {
                    rotate_left_with_kick(&mut now, &board);
                    }},
            'l' => {if now.kind != Mino::O {
                    rotate_right_with_kick(&mut now, &board);
                    }},
            'j' => {board.hard_drop(&mut now);
                    hold.can_hold = true;
                    now = Now::init_with(next.give_block());},
            's' => {move_down(&mut now, &board);},
            'q' => {println!("game quit");
                    clear_one();
                    game_manager.print_record();
                    thread::sleep(Duration::from_secs(5));
                    break},
            'w' => {hold.holding(&mut now, &mut next);},
            'e' => {
                    clear_one();
                    println!("game is stoped");
                    println!(" ===조작법===");
                    println!(" a = 왼쪽 이동, d = 오른쪽 이동 \n s = 소프트 드랍, j = 하드 드랍");
                    println!(" k = 왼쪽 회전, l = 오른쪽 회전 \n w = 홀드, q = 종료");
                    println!("   재개 하려면 <e>를 누르세요");
                    loop {
                        let pause = input();
                            if pause == 'e' {
                                clear_one();
                                break;
                            }
                            sleep_ms(100);
                        }
                    continue
                    },
             _  => {},
        };

        if let Some(start) = lock_timer {
            if Instant::now().duration_since(start) > Duration::from_millis(2000) {
                board.fix_with(&now);
                hold.can_hold = true;
                now = Now::init_with(next.give_block());

                lock_timer = None;
            }
        }


        if Instant::now().duration_since(last_tick) >= game_manager.tick_rate {
            let moved = move_down(&mut now, &board);

            if moved {
                lock_timer = None; // 움직였으니 유예시간 초기화
            } else {
                if lock_timer.is_none() {
                    lock_timer = Some(Instant::now()); // 유예시간 시작
                }
            }

            last_tick = Instant::now();
        }
        let cleared_line = board.clear_line() as u32;
        if cleared_line > 0 {
            game_manager.cleard += cleared_line;
            game_manager.score_up(cleared_line);
            game_manager.level_up();
        }

        if now.loc == ORIGIN_LOC {
            if !board.is_possible(&now) {
                    clear_one();
                    println!("GAME OVER....");
                    game_manager.print_record();

                    thread::sleep(Duration::from_secs(3));
                    return
                }
        }
        print_all(&board.draw_with(&now), &hold, &next, &game_manager);

        clear();
    }

    //종료 시 커서 복원
    execute!(stdout(), Show).unwrap();
}
//rotate함수에 벽킥 구현
//타이머 기능 구현 -> 자동 낙하, 자동 픽스