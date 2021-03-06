use crate::tetris_game::*;
use storm::time::*;
use storm::*;

pub struct TetrisState {
    is_active: bool,
    generate_new_cluster: bool,
    update_count: u32,
    engine: Engine,
    board: [[TetrisBlockType; 10]; 40],
    current_cluster: TetrisCluster,
    score: i32,
    total_lines_cleared: u32,
    lateral_move: bool,
    movement_vector: Pos,
    rotation_direction: i32,
    sprites: Vec<Sprite>,
    strings: Vec<Text>,
    screen: BatchToken,
    clock: Clock,
    is_paused: bool,
    audio: Bruback,
    main_sink: SinkID,
    effect_sink: SinkID,
}

impl TetrisState {
    pub fn new(mut engine: Engine) -> TetrisState {
        let mut board = [[TetrisBlockType::Empty; 10]; 40];
        engine.window_clear_color(storm::color::BLACK);

        let screen = engine.batch_create(&BatchSettings::default());

        let mut sprites = Vec::new();
        let clock = Clock::new(144);

        let current_cluster = TetrisCluster::new(Pos::new(4, 38), TetrisBlockType::random_tetris_block());

        let mut sprite = Sprite::default();
        sprite.size.x = sprite.size.x / 5;
        sprite.size.y = sprite.size.y / 5;

        for x in 0..10 {
            for y in 0..40 {
                sprite.pos.x = (x * 20) as f32 - 100.0f32;
                sprite.pos.y = (y * 20) as f32 - 400.0f32;
                sprites.push(sprite);
            }
        }

        let mut strings = Vec::new();
        let mut text = Text::default();

        {
            text.set_string("Score: 0");
            text.color = color::WHITE;
            text.pos.x = 125.0;
            text.pos.y += 375.0;
            strings.push(text);
            // Assign the strings we want to draw to a batch.

            let mut menu_text = Text::default();
            //test string for menu text
            menu_text.set_string("Hey");
            menu_text.color = color::WHITE;
            menu_text.pos.x = 0.0;
            menu_text.pos.y = 0.0;
            strings.push(menu_text);

            engine.text_set(&screen, &strings);
        }

        let position = current_cluster.current_position;

        for offset in current_cluster.offsets.iter() {
            let block_pos = position + *offset;
            board[block_pos.y as usize][block_pos.x as usize] = current_cluster.block_type;
        }

        engine.sprite_set(&screen, &sprites);

        let mut bruback = Bruback::new();

        let main_sink = bruback.create_new_sink();

        bruback.set_track_volume(0.05, main_sink);
        bruback.play_track(String::from("examples/resources/tetris.ogg"), main_sink);

        let effect_sink = bruback.create_new_sink();

        TetrisState {
            is_active: true,
            generate_new_cluster: false,
            update_count: 0,
            engine,
            board,
            current_cluster,
            score: 0,
            total_lines_cleared: 0,
            lateral_move: false,
            movement_vector: Pos::new(0, 0),
            rotation_direction: 0,
            sprites,
            strings,
            screen,
            clock,
            is_paused: false,
            audio: bruback,
            main_sink,
            effect_sink,
        }
    }

    pub fn update(&mut self) {
        while self.is_active {
            self.lateral_move = false;
            self.rotation_direction = 0;

            if self.generate_new_cluster == true {
                self.current_cluster =
                    TetrisCluster::new(Pos::new(4, 38), TetrisBlockType::random_tetris_block());
                self.generate_new_cluster = false;
            }

            self.check_input();

            if self.is_paused == false {
                if self.update_count == (20 - self.total_lines_cleared / 10)
                    || self.lateral_move
                    || self.rotation_direction != 0
                {
                    self.attempt_to_move_block();
                } else {
                    self.update_count += 1;
                }
            }

            self.set_color_of_board();

            if self.generate_new_cluster {
                self.read_and_clear_map();
            }

            if self.is_paused == false {
                self.engine.sprite_set(&self.screen, &self.sprites);
            } else {
                self.draw_menu_text();
            }

            self.engine.window_commit();
            self.clock.tick();
        }
    }

    pub fn draw_menu_text(&mut self) {
        self.strings[1].set_string(&"Hey");
        self.engine.text_set(&self.screen, &self.strings);
    }

    pub fn read_and_clear_map(&mut self) {
        let mut row = 0;
        let mut cleared_rows = 0;
        while row != 40 {
            let mut has_empty_slot = false;
            for x in 0..10 {
                match self.board[row][x] {
                    TetrisBlockType::Empty => {
                        has_empty_slot = true;
                    }
                    _ => {}
                }
            }

            if has_empty_slot == false {
                cleared_rows += 1;
                for x in 0..10 {
                    self.board[row][x] = TetrisBlockType::Empty;
                }
                let mut shift_row = row + 1;
                while shift_row != 40 {
                    for x in 0..10 {
                        self.board[shift_row - 1][x] = self.board[shift_row][x];
                        self.board[shift_row][x] = TetrisBlockType::Empty;
                    }
                    shift_row += 1;
                }
                self.score += 100;
            } else {
                row += 1;
            }
        }

        if cleared_rows != 0 {
            self.audio.set_track_volume(0.05, self.effect_sink);
            self.audio.play_track(String::from("examples/resources/clear.wav"), self.effect_sink);
        }
        self.total_lines_cleared += cleared_rows;

        self.strings[0].set_string(&("Score".to_string() + " : " + &self.score.to_string()));
        self.engine.text_set(&self.screen, &self.strings);
    }

    pub fn set_color_of_board(&mut self) {
        for x in 0..10 {
            for y in 0..40 {
                let index = x * 40 + y;
                self.sprites[index].color = self.board[y][x].color();
            }
        }
    }

    pub fn attempt_to_move_block(&mut self) {
        let mut position = self.current_cluster.current_position;

        //test the set of board positions under the current ones, are they occupied/the end of the board
        //first we need to erase of current postions so we don't set off the check
        for offset in self.current_cluster.offsets.iter() {
            let block_pos = position + *offset;
            self.board[block_pos.y as usize][block_pos.x as usize] = TetrisBlockType::Empty;
        }

        //check if we can do the move
        let mut all_empty = true;
        let mut hit_edge = false;

        let use_offsets;
        if self.rotation_direction == 0 {
            use_offsets = self.current_cluster.offsets;
        } else {
            use_offsets = self.current_cluster.generate_offsets(self.rotation_direction);
        }

        for offset in use_offsets.iter() {
            let block_pos = position + *offset + self.movement_vector;
            if block_pos.x < 0 || block_pos.y < 0 || block_pos.x == 10 || block_pos.y == 40 {
                hit_edge = true;
                continue;
            }
            match self.board[block_pos.y as usize][block_pos.x as usize] {
                TetrisBlockType::Empty => {}
                _ => {
                    all_empty = false;
                }
            }
        }

        if hit_edge == false && all_empty {
            self.current_cluster.offsets = use_offsets;
        }

        if hit_edge || (self.movement_vector.y != 0 && all_empty == false) {
            self.movement_vector = Pos::new(0, 0);
        }

        //if we passed the check, update the position of the block
        if all_empty {
            position = position + self.movement_vector;
            self.current_cluster.current_position = position;
        } else if hit_edge == false && self.lateral_move == false {
            self.generate_new_cluster = true;
        }

        //write the postion back into the board into either the new or old place
        for offset in self.current_cluster.offsets.iter() {
            let block_pos = position + *offset;
            self.board[block_pos.y as usize][block_pos.x as usize] = self.current_cluster.block_type;
            if block_pos.y == 0 {
                self.generate_new_cluster = true;
            }
        }
        if self.update_count == (20 - self.total_lines_cleared / 10) {
            self.update_count = 0;
        }
    }

    pub fn check_input(&mut self) {
        self.movement_vector.y = -1;
        self.movement_vector.x = 0;
        while let Some(message) = self.engine.input_poll() {
            match message {
                InputMessage::CloseRequested => self.is_active = false,
                InputMessage::KeyPressed(key) => match key {
                    KeyboardButton::Left => {
                        self.movement_vector.x = -1;
                        self.movement_vector.y = 0;
                        self.lateral_move = true;
                    }
                    KeyboardButton::Right => {
                        self.movement_vector.x = 1;
                        self.movement_vector.y = 0;
                        self.lateral_move = true;
                    }
                    KeyboardButton::Q => {
                        self.rotation_direction = -1;
                        self.movement_vector.x = 0;
                        self.movement_vector.y = 0;
                    }
                    KeyboardButton::E => {
                        self.rotation_direction = 1;
                        self.movement_vector.x = 0;
                        self.movement_vector.y = 0;
                    }
                    KeyboardButton::P => {
                        self.is_paused = !self.is_paused;
                        if self.is_paused {
                            self.engine.sprite_clear(&self.screen);
                        }
                    }

                    KeyboardButton::Escape => self.is_active = false,
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
