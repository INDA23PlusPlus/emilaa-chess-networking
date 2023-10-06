mod shader;
mod model;

use std::collections::HashMap;
use std::{env, thread};
use std::io::Read;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use model::*;
use shader::*;

use glfw::*;
use glm::*;
use freetype::Library;
use freetype::face::LoadFlag;
use chess_network_protocol::*;
use serde::Deserialize;

#[derive(PartialEq)]
enum HostState {
    WaitingForCon,
    HostPlaying,
    ClientPlaying,
    TestingClientMove,
    BadClientMove,
    RespondToClient,
    SendToClient,
    PromotingHost,
    PromotingClient
}

#[derive(PartialEq)]
enum ClientState {
    Pending,
    HostPlaying,
    ClientPlaying,
    WaitingForResponse,
}

struct HostGame {
    chess: ludviggl_chess::Game,
    board: [Model2D; 64],
    white_pieces: [Model2D; 6],
    black_pieces: [Model2D; 6],
    game_end: bool,
    promoting: bool,
    selected_prom: ludviggl_chess::Piece,
    host_turn: bool,
    recieved_move: (usize, usize, usize, usize),
    made_move: (usize, usize, usize, usize),
    client_connected: bool,
    state: HostState
}

struct ClientGame {
    board: [Model2D; 64],
    white_pieces: [Model2D; 6],
    black_pieces: [Model2D; 6],
    white_pos: Vec<(usize, usize, usize)>,
    black_pos: Vec<(usize, usize, usize)>,
    game_end: bool,
    promoting: bool,
    _selected_prom: chess_network_protocol::Piece,
    host_turn: bool,
    m_from: (usize, usize),
    m_to: (usize, usize),
    send_move: bool,
    state: ClientState
}

struct Character {
    texture_id: u32,
    size: IVec2,
    bearing: IVec2,
    advance: u32
}

impl HostGame {
    pub fn new() -> HostGame {
        let mut b: [Model2D; 64] = (0..64).map(|_| Model2D::dummy()).collect::<Vec<_>>().try_into().unwrap();
        let wp: [Model2D; 6] = (0..6i8).map(|i| Model2D::white_piece(i)).collect::<Vec<_>>().try_into().unwrap();
        let bp: [Model2D; 6] = (0..6i8).map(|i| Model2D::black_piece(i)).collect::<Vec<_>>().try_into().unwrap();

        for y in 0..8usize {
            for x in 0..8usize {
                if y % 2 == 0 {
                    if x % 2 == 0 {
                        b[y*8 + x] = Model2D::white_tile();
                    } else {
                        b[y*8 + x] = Model2D::black_tile();
                    }
                } else {
                    if (x + 1) % 2 == 0 {
                        b[y*8 + x] = Model2D::white_tile();
                    } else {
                        b[y*8 + x] = Model2D::black_tile();
                    }                   
                }
                
                b[y*8 + x].transform.translation.x = x as f32 - 3.5;
                b[y*8 + x].transform.translation.y = -1.0 * (-(y as f32) + 3.5);
            }
        }
    
        return HostGame{ 
            chess: ludviggl_chess::Game::new(), 
            board: b,
            white_pieces: wp,
            black_pieces: bp,
            game_end: false,
            promoting: false,
            selected_prom: ludviggl_chess::Piece::Pawn,
            host_turn: true,
            recieved_move: (usize::MAX, usize::MAX, usize::MAX, usize::MAX),
            made_move: (usize::MAX, usize::MAX, usize::MAX, usize::MAX),
            client_connected: false,
            state: HostState::WaitingForCon
        };
    }
}

impl ClientGame {
    pub fn new() -> ClientGame {
        let mut b: [Model2D; 64] = (0..64).map(|_| Model2D::dummy()).collect::<Vec<_>>().try_into().unwrap();
        let wp: [Model2D; 6] = (0..6i8).map(|i| Model2D::white_piece(i)).collect::<Vec<_>>().try_into().unwrap();
        let bp: [Model2D; 6] = (0..6i8).map(|i| Model2D::black_piece(i)).collect::<Vec<_>>().try_into().unwrap();

        for y in 0..8usize {
            for x in 0..8usize {
                if y % 2 == 0 {
                    if x % 2 == 0 {
                        b[y*8 + x] = Model2D::white_tile();
                    } else {
                        b[y*8 + x] = Model2D::black_tile();
                    }
                } else {
                    if (x + 1) % 2 == 0 {
                        b[y*8 + x] = Model2D::white_tile();
                    } else {
                        b[y*8 + x] = Model2D::black_tile();
                    }                   
                }
                
                b[y*8 + x].transform.translation.x = x as f32 - 3.5;
                b[y*8 + x].transform.translation.y = -1.0 * (-(y as f32) + 3.5);
            }
        }
    
        return ClientGame{
            board: b,
            white_pieces: wp,
            black_pieces: bp,
            black_pos: vec![],
            white_pos: vec![],
            game_end: false,
            promoting: false,
            _selected_prom: chess_network_protocol::Piece::None,
            host_turn: true,
            m_from: (usize::MAX, usize::MAX),
            m_to: (usize::MAX, usize::MAX),
            send_move: false,
            state: ClientState::Pending
        };
    }
}

fn main() {
    let host: bool;
    let address: String;
    let c_white: bool;
    let args: Vec<String> = env::args().collect::<Vec<String>>();
    if args.len() > 1 {
        host = false;
        address = args[1].clone();
    } else {
        host = true;
        address = "".to_string();
    }

    if args.len() > 2 {
        c_white = if args[2].to_lowercase() == "b" { false } else { true };
    } else {
        c_white = true;
    }

    let mut glfw = init(fail_on_errors!()).unwrap();

    glfw.window_hint(WindowHint::Resizable(false));
    glfw.window_hint(WindowHint::ContextVersionMajor(4));
    glfw.window_hint(WindowHint::ContextVersionMinor(6));
    
    let (mut window, events) = glfw.create_window(
        1000, 
        800, 
        "Chess game", 
        WindowMode::Windowed).expect("Failed to create window..."
    );
    
    window.make_current();
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    gl::load_with(|s| window.get_proc_address(s));

    let ftlib = Library::init().unwrap();
    let face = ftlib.new_face("assets/comic.ttf", 0).unwrap();
    face.set_pixel_sizes(0, 48).unwrap();
    let mut characters: HashMap<char, Character> = HashMap::new();
    load_ttf(&mut characters, face);
    let mut char_quad = Model2D::character();
    
    let proj = orthographic_projection(6.0, -4.0, 4.0, -4.0, 1.0, -1.0);
    let text_proj = orthographic_projection(1000.0, 0.0, 800.0, 0.0, 1.0, -1.0);

    let tile_shader = Shader::new("shaders/tile.vert", "shaders/tile.frag");
    let piece_shader = Shader::new("shaders/piece.vert", "shaders/piece.frag");
    let text_shader = Shader::new("shaders/text.vert", "shaders/text.frag");

    let sprites = load_texture("assets/sprites.png");

    unsafe {
    gl::Viewport(0, 0, 1000, 800);
    gl::Enable(gl::BLEND);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    // We are connecting.
    if !host {
        let game_ = Arc::new(Mutex::new(ClientGame::new()));
        let render = Arc::clone(&game_);
        let network = Arc::clone(&game_);

        thread::spawn(move || {
            let stream = TcpStream::connect(address).unwrap();
            let mut de = serde_json::Deserializer::from_reader(&stream);

            let handshake = ClientToServerHandshake {
                server_color: if c_white { Color::Black } else { Color::White }
            };
        
            serde_json::to_writer(&stream, &handshake).unwrap();

            let des = ServerToClientHandshake::deserialize(&mut de).unwrap();

            let mut game = network.lock().unwrap();

            game.state = if c_white { ClientState::ClientPlaying } else { ClientState::HostPlaying };
            game.host_turn = if c_white { false } else { true };

            for y in 0..8 {
                for x in 0..8 {
                    if (des.board[y][x] as i8) < 6 {
                        if des.board[y][x] as i8 == 1 { game.black_pos.push((x, y, 2)); }
                        else if des.board[y][x] as i8 == 2 { game.black_pos.push((x, y, 3)); }
                        else if des.board[y][x] as i8 == 3 { game.black_pos.push((x, y, 1)); }
                        else { game.black_pos.push((x, y, des.board[y][x] as usize)); }
                    }

                    if (des.board[y][x] as i8) < 12 && des.board[y][x] as i8 > 5 {
                        if des.board[y][x] as i8 == 7 { game.white_pos.push((x, y, 2)); }
                        else if des.board[y][x] as i8 == 8 { game.white_pos.push((x, y, 3)); }
                        else if des.board[y][x] as i8 == 9 { game.white_pos.push((x, y, 1)); }
                        else { game.white_pos.push((x, y, des.board[y][x] as usize - 6)); }
                    }
                }
            }

            std::mem::drop(game);

            loop {
                let mut game = network.lock().unwrap();
                
                match game.state {
                    ClientState::ClientPlaying => {
                        if game.send_move {
                            let m = Move{ start_x: game.m_from.0, start_y: game.m_from.1, end_x: game.m_to.0, end_y: game.m_to.1, promotion: Piece::None };
                            let move_to_send = ClientToServer::Move(m);
                            serde_json::to_writer(&stream, &move_to_send).unwrap();
                            game.send_move = false;
                            game.m_from = (usize::MAX, usize::MAX);
                            game.m_to = (usize::MAX, usize::MAX);
                            game.state = ClientState::WaitingForResponse;
                        }
                    }

                    ClientState::WaitingForResponse => {
                        std::mem::drop(game);
                        let res = ServerToClient::deserialize(&mut de).unwrap();

                        game = network.lock().unwrap();

                        match res {
                            ServerToClient::State { board: b, moves: _, joever: j, move_made: _ } => {
                                game.black_pos.clear();
                                game.white_pos.clear();
                                
                                for y in 0..8 {
                                    for x in 0..8 {
                                        if (b[y][x] as i8) < 6 {
                                            if b[y][x] as i8 == 1 { game.black_pos.push((x, y, 2)); }
                                            else if b[y][x] as i8 == 2 { game.black_pos.push((x, y, 3)); }
                                            else if b[y][x] as i8 == 3 { game.black_pos.push((x, y, 1)); }
                                            else { game.black_pos.push((x, y, b[y][x] as usize)); }
                                        }
                    
                                        if (b[y][x] as i8) < 12 && b[y][x] as i8 > 5 {
                                            if b[y][x] as i8 == 7 { game.white_pos.push((x, y, 2)); }
                                            else if b[y][x] as i8 == 8 { game.white_pos.push((x, y, 3)); }
                                            else if b[y][x] as i8 == 9 { game.white_pos.push((x, y, 1)); }
                                            else { game.white_pos.push((x, y, b[y][x] as usize - 6)); }
                                        }
                                    }
                                }
                                
                                game.game_end = if j != Joever::Ongoing { true } else { false };
                                game.host_turn = true;

                                game.state = ClientState::HostPlaying;
                            }

                            ServerToClient::Error { board: b, moves: _, joever: _, message: _ } => {
                                for y in 0..8 {
                                    for x in 0..8 {
                                        if (b[y][x] as i8) < 6 {
                                            if b[y][x] as i8 == 1 { game.black_pos.push((x, y, 2)); }
                                            else if b[y][x] as i8 == 2 { game.black_pos.push((x, y, 3)); }
                                            else if b[y][x] as i8 == 3 { game.black_pos.push((x, y, 1)); }
                                            else { game.black_pos.push((x, y, b[y][x] as usize)); }
                                        }
                    
                                        if (b[y][x] as i8) < 12 && b[y][x] as i8 > 5 {
                                            if b[y][x] as i8 == 7 { game.white_pos.push((x, y, 2)); }
                                            else if b[y][x] as i8 == 8 { game.white_pos.push((x, y, 3)); }
                                            else if b[y][x] as i8 == 9 { game.white_pos.push((x, y, 1)); }
                                            else { game.white_pos.push((x, y, b[y][x] as usize - 6)); }
                                        }
                                    }
                                }

                                game.state = ClientState::ClientPlaying;
                            }

                            _ => { }
                        }
                    }

                    ClientState::HostPlaying => {
                        std::mem::drop(game);
                        let res = ServerToClient::deserialize(&mut de).unwrap();

                        game = network.lock().unwrap();

                        match res {
                            ServerToClient::State { board: b, moves: _, joever: j, move_made: _ } => {
                                game.black_pos.clear();
                                game.white_pos.clear();
                                
                                for y in 0..8 {
                                    for x in 0..8 {
                                        if (b[y][x] as i8) < 6 {
                                            if b[y][x] as i8 == 1 { game.black_pos.push((x, y, 2)); }
                                            else if b[y][x] as i8 == 2 { game.black_pos.push((x, y, 3)); }
                                            else if b[y][x] as i8 == 3 { game.black_pos.push((x, y, 1)); }
                                            else { game.black_pos.push((x, y, b[y][x] as usize)); }
                                        }
                    
                                        if (b[y][x] as i8) < 12 && b[y][x] as i8 > 5 {
                                            if b[y][x] as i8 == 7 { game.white_pos.push((x, y, 2)); }
                                            else if b[y][x] as i8 == 8 { game.white_pos.push((x, y, 3)); }
                                            else if b[y][x] as i8 == 9 { game.white_pos.push((x, y, 1)); }
                                            else { game.white_pos.push((x, y, b[y][x] as usize - 6)); }
                                        }
                                    }
                                }
                                
                                game.game_end = if j != Joever::Ongoing { true } else { false };
                                game.host_turn = false;

                                game.state = ClientState::ClientPlaying;
                            }

                            _ => { }
                        }
                    }

                    _ => { }
                }
                
                std::mem::drop(game);
                std::thread::sleep(std::time::Duration::from_millis(7));
            }
        });

        while !window.should_close() {
            let mut game = render.lock().unwrap();

            unsafe {
                gl::ClearColor(0.3, 0.3, 0.2, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                
                tile_shader.use_program();
                tile_shader.set_mat4("projection", proj);
                for i in 0..64 {
                    game.board[i].draw(&tile_shader);
                }

                piece_shader.use_program();
                piece_shader.set_mat4("projection", proj);
                gl::BindTexture(gl::TEXTURE_2D, sprites);
                for (x, y, piece) in game.white_pos.to_owned() {
                    game.white_pieces[piece as usize].transform.translation.x = x as f32 - 3.5;
                    game.white_pieces[piece as usize].transform.translation.y = -1.0 * (-(y as f32) + 3.5);
                    game.white_pieces[piece as usize].draw(&piece_shader);
                }

                for (x, y, piece) in game.black_pos.to_owned() {
                    game.black_pieces[piece as usize].transform.translation.x = x as f32 - 3.5;
                    game.black_pieces[piece as usize].transform.translation.y = -1.0 * (-(y as f32) + 3.5);
                    game.black_pieces[piece as usize].draw(&piece_shader);
                }
                gl::BindTexture(gl::TEXTURE_2D, 0);

                text_shader.use_program();
                text_shader.set_mat4("projection", text_proj);
                if game.game_end {
                    let winner = if game.host_turn { "You lose!" } else { "You win!" };
                    render_text(&text_shader, winner.to_string(), 800.0, 640.0, 0.7, vec4(1.0, 1.0, 1.0, 1.0), &characters, &mut char_quad);
                    render_text(&text_shader, "Press \'ESC\' to exit.".to_string(), 800.0, 610.0, 0.44, vec4(1.0, 1.0, 1.0, 1.0), &characters, &mut char_quad);
                } else {
                    let turn = if game.host_turn { "Host is playing." } else { "You are playing." };
                    render_text(&text_shader, turn.to_string(),  800.0, 640.0, 0.5, vec4(1.0, 1.0, 1.0, 1.0), &characters, &mut char_quad);
                }
            }

            // match game.chess.get_state() {
            //     ludviggl_chess::State::CheckMate => {
            //         game.game_end = true;
            //     }

            //     ludviggl_chess::State::SelectPromotion => {
            //         game.promoting = true;
            //         text_shader.use_program();
            //         text_shader.set_mat4("projection", text_proj);
            //         render_text(&text_shader, "Select promotion:".to_string(), 800.0, 600.0, 0.5, vec4(1.0, 1.0, 1.0, 1.0), &characters, &mut char_quad);
            //         render_text(&text_shader, "1: Rook".to_string(), 800.0, 570.0, 0.6, vec4(1.0, 1.0, 1.0, 1.0), &characters, &mut char_quad);
            //         render_text(&text_shader, "2: Knight".to_string(), 800.0, 540.0, 0.6, vec4(1.0, 1.0, 1.0, 1.0), &characters, &mut char_quad);
            //         render_text(&text_shader, "3: Bishop".to_string(), 800.0, 510.0, 0.6, vec4(1.0, 1.0, 1.0, 1.0), &characters, &mut char_quad);
            //         render_text(&text_shader, "4: Queen".to_string(), 800.0, 480.0, 0.6, vec4(1.0, 1.0, 1.0, 1.0), &characters, &mut char_quad);

            //         let prom = game.selected_prom;
            //         if game.chess.select_promotion(prom).is_ok() {
            //             game.promoting = false;
            //             game.selected_prom = ludviggl_chess::Piece::Pawn;
            //         }
            //     }

            //     _ => { }
            // }

            for (_, event) in flush_messages(&events) {
                match event {
                    WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                        window.set_should_close(true);
                    }

                    WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                        if !game.game_end && !game.promoting && game.state == ClientState::ClientPlaying {
                            client_on_pick(&mut game, &window);
                        }
                    }

                    _ => {}
                }

                // if game.promoting && game.host_turn {
                //     match event {
                //         WindowEvent::Key(Key::Num1, _, Action::Press, _) => { game.selected_prom = ludviggl_chess::Piece::Rook; }
                //         WindowEvent::Key(Key::Num2, _, Action::Press, _) => { game.selected_prom = ludviggl_chess::Piece::Knight; }
                //         WindowEvent::Key(Key::Num3, _, Action::Press, _) => { game.selected_prom = ludviggl_chess::Piece::Bishop; }
                //         WindowEvent::Key(Key::Num4, _, Action::Press, _) => { game.selected_prom = ludviggl_chess::Piece::Queen; }

                //         _ => { }
                //     }
                // }
            }

            window.swap_buffers();
            glfw.poll_events();

            std::mem::drop(game);
            std::thread::sleep(std::time::Duration::from_millis(7));
        }

    } else {

        // We are hosting.
        let game_ = Arc::new(Mutex::new(HostGame::new()));
        let render = Arc::clone(&game_);
        let network = Arc::clone(&game_);
        
        thread::spawn(move || {
            let listener = TcpListener::bind("127.0.0.1:8384").unwrap();
            let (stream, _addr) = listener.accept().unwrap();
            
            let mut de = serde_json::Deserializer::from_reader(&stream);
            let des = ClientToServerHandshake::deserialize(&mut de).unwrap();
            
            let mut game = network.lock().unwrap();

            game.host_turn = if des.server_color == Color::White { true } else { false };
            game.client_connected = true;
            game.state = if game.host_turn { HostState::HostPlaying } else { HostState::ClientPlaying };

            let mut board_copy = [[Piece::None; 8]; 8];
            for (p, x, y) in game.chess.get_black_positions().to_owned() {
                board_copy[y as usize][x as usize] = match p {
                    ludviggl_chess::Piece::Pawn => { Piece::BlackPawn }
                    ludviggl_chess::Piece::Rook => { Piece::BlackRook }
                    ludviggl_chess::Piece::Knight => { Piece::BlackKnight }
                    ludviggl_chess::Piece::Bishop => { Piece::BlackBishop }
                    ludviggl_chess::Piece::King => { Piece::BlackKing }
                    ludviggl_chess::Piece::Queen => { Piece::BlackQueen }
                }
            }

            for (p, x, y) in game.chess.get_white_positions().to_owned() {
                board_copy[y as usize][x as usize] = match p {
                    ludviggl_chess::Piece::Pawn => { Piece::WhitePawn }
                    ludviggl_chess::Piece::Rook => { Piece::WhiteRook }
                    ludviggl_chess::Piece::Knight => { Piece::WhiteKnight }
                    ludviggl_chess::Piece::Bishop => { Piece::WhiteBishop }
                    ludviggl_chess::Piece::King => { Piece::WhiteKing }
                    ludviggl_chess::Piece::Queen => { Piece::WhiteQueen }
                }
            }

            std::mem::drop(game);

            let handshake = ServerToClientHandshake {
                features: vec![
                    Features::EnPassant, 
                    Features::Castling
                    ],
                board: board_copy,
                moves: vec![],
                joever: Joever::Ongoing
            };
        
            serde_json::to_writer(&stream, &handshake).unwrap();

            loop {
                let mut game = network.lock().unwrap();
                
                match game.state {
                    HostState::ClientPlaying => {
                        std::mem::drop(game);
                        let res = ClientToServer::deserialize(&mut de).unwrap().to_owned();
                        
                        game = network.lock().unwrap();

                        match res {
                            ClientToServer::Draw => { game.game_end = true; }
                            ClientToServer::Resign => { game.game_end = true; }
                            ClientToServer::Move(Move{ start_x: sx, start_y: sy, end_x: ex, end_y: ey, promotion: _ }) => {
                                game.recieved_move = (sx, sy, ex, ey);
                                game.state = HostState::TestingClientMove;
                            }
                        }
                    }
                    
                    HostState::RespondToClient => {
                        let mut board_copy = [[Piece::None; 8]; 8];
                        for (p, x, y) in game.chess.get_black_positions().to_owned() {
                            board_copy[y as usize][x as usize] = match p {
                                ludviggl_chess::Piece::Pawn => { Piece::BlackPawn }
                                ludviggl_chess::Piece::Rook => { Piece::BlackRook }
                                ludviggl_chess::Piece::Knight => { Piece::BlackKnight }
                                ludviggl_chess::Piece::Bishop => { Piece::BlackBishop }
                                ludviggl_chess::Piece::King => { Piece::BlackKing }
                                ludviggl_chess::Piece::Queen => { Piece::BlackQueen }
                            }
                        }
                    
                        for (p, x, y) in game.chess.get_white_positions().to_owned() {
                            board_copy[y as usize][x as usize] = match p {
                                ludviggl_chess::Piece::Pawn => { Piece::WhitePawn }
                                ludviggl_chess::Piece::Rook => { Piece::WhiteRook }
                                ludviggl_chess::Piece::Knight => { Piece::WhiteKnight }
                                ludviggl_chess::Piece::Bishop => { Piece::WhiteBishop }
                                ludviggl_chess::Piece::King => { Piece::WhiteKing }
                                ludviggl_chess::Piece::Queen => { Piece::WhiteQueen }
                            }
                        }

                        let send = ServerToClient::State { 
                            board: board_copy, 
                            moves: vec![], 
                            joever: if game.game_end { Joever::Indeterminate } else { Joever::Ongoing }, 
                            move_made: Move {
                                start_x: game.recieved_move.0,
                                start_y: game.recieved_move.1,
                                end_x: game.recieved_move.2,
                                end_y: game.recieved_move.3,
                                promotion: Piece::None
                            }
                        };

                        serde_json::to_writer(&stream, &send).unwrap();

                        game.recieved_move = (0,0,0,0);
                        game.host_turn = true;

                        game.state =  HostState::HostPlaying;
                    }

                    HostState::BadClientMove => {
                        let mut board_copy = [[Piece::None; 8]; 8];
                        for (p, x, y) in game.chess.get_black_positions().to_owned() {
                            board_copy[y as usize][x as usize] = match p {
                                ludviggl_chess::Piece::Pawn => { Piece::BlackPawn }
                                ludviggl_chess::Piece::Rook => { Piece::BlackRook }
                                ludviggl_chess::Piece::Knight => { Piece::BlackKnight }
                                ludviggl_chess::Piece::Bishop => { Piece::BlackBishop }
                                ludviggl_chess::Piece::King => { Piece::BlackKing }
                                ludviggl_chess::Piece::Queen => { Piece::BlackQueen }
                            }
                        }
                    
                        for (p, x, y) in game.chess.get_white_positions().to_owned() {
                            board_copy[y as usize][x as usize] = match p {
                                ludviggl_chess::Piece::Pawn => { Piece::WhitePawn }
                                ludviggl_chess::Piece::Rook => { Piece::WhiteRook }
                                ludviggl_chess::Piece::Knight => { Piece::WhiteKnight }
                                ludviggl_chess::Piece::Bishop => { Piece::WhiteBishop }
                                ludviggl_chess::Piece::King => { Piece::WhiteKing }
                                ludviggl_chess::Piece::Queen => { Piece::WhiteQueen }
                            }
                        }

                        let send = ServerToClient::Error { 
                            board: board_copy, 
                            moves: vec![], 
                            joever: if game.game_end { Joever::Indeterminate } else { Joever::Ongoing }, 
                            message: "Bad move.".to_string()
                        };

                        serde_json::to_writer(&stream, &send).unwrap();
                        game.state = HostState::ClientPlaying;
                    }

                    HostState::SendToClient => {
                        let mut board_copy = [[Piece::None; 8]; 8];

                        for (p, x, y) in game.chess.get_black_positions().to_owned() {
                            board_copy[y as usize][x as usize] = match p {
                                ludviggl_chess::Piece::Pawn => { Piece::BlackPawn }
                                ludviggl_chess::Piece::Rook => { Piece::BlackRook }
                                ludviggl_chess::Piece::Knight => { Piece::BlackKnight }
                                ludviggl_chess::Piece::Bishop => { Piece::BlackBishop }
                                ludviggl_chess::Piece::King => { Piece::BlackKing }
                                ludviggl_chess::Piece::Queen => { Piece::BlackQueen }
                            }
                        }
                    
                        for (p, x, y) in game.chess.get_white_positions().to_owned() {
                            board_copy[y as usize][x as usize] = match p {
                                ludviggl_chess::Piece::Pawn => { Piece::WhitePawn }
                                ludviggl_chess::Piece::Rook => { Piece::WhiteRook }
                                ludviggl_chess::Piece::Knight => { Piece::WhiteKnight }
                                ludviggl_chess::Piece::Bishop => { Piece::WhiteBishop }
                                ludviggl_chess::Piece::King => { Piece::WhiteKing }
                                ludviggl_chess::Piece::Queen => { Piece::WhiteQueen }
                            }
                        }

                        let send = ServerToClient::State { 
                            board: board_copy, 
                            moves: vec![], 
                            joever: if game.game_end { Joever::Indeterminate } else { Joever::Ongoing }, 
                            move_made: Move {
                                start_x: game.made_move.0,
                                start_y: game.made_move.1,
                                end_x: game.made_move.2,
                                end_y: game.made_move.3,
                                promotion: Piece::None
                            }
                        };

                        serde_json::to_writer(&stream, &send).unwrap();
                        game.made_move = (usize::MAX, usize::MAX, usize::MAX, usize::MAX);
                        game.host_turn = false;

                        game.state =  HostState::ClientPlaying;
                    }

                    _ => { }
                }

                std::mem::drop(game);
                std::thread::sleep(std::time::Duration::from_millis(7));
            }
        });

        while !window.should_close() {
            let mut game = render.lock().unwrap();

            unsafe {
                gl::ClearColor(0.3, 0.3, 0.2, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                
                tile_shader.use_program();
                tile_shader.set_mat4("projection", proj);
                for i in 0..64 {
                    game.board[i].draw(&tile_shader);
                }

                piece_shader.use_program();
                piece_shader.set_mat4("projection", proj);
                gl::BindTexture(gl::TEXTURE_2D, sprites);
                for (piece, x, y) in game.chess.get_white_positions().to_owned() {
                    game.white_pieces[piece as usize].transform.translation.x = x as f32 - 3.5;
                    game.white_pieces[piece as usize].transform.translation.y = -1.0 * (-(y as f32) + 3.5);
                    game.white_pieces[piece as usize].draw(&piece_shader);
                }

                for (piece, x, y) in game.chess.get_black_positions().to_owned() {
                    game.black_pieces[piece as usize].transform.translation.x = x as f32 - 3.5;
                    game.black_pieces[piece as usize].transform.translation.y = -1.0 * (-(y as f32) + 3.5);
                    game.black_pieces[piece as usize].draw(&piece_shader);
                }
                gl::BindTexture(gl::TEXTURE_2D, 0);

                text_shader.use_program();
                text_shader.set_mat4("projection", text_proj);
                if game.game_end {
                    let winner = if game.host_turn { "You lose!" } else { "You win!" };
                    render_text(&text_shader, winner.to_string(), 800.0, 640.0, 0.7, vec4(1.0, 1.0, 1.0, 1.0), &characters, &mut char_quad);
                    render_text(&text_shader, "Press \'ESC\' to exit.".to_string(), 800.0, 610.0, 0.44, vec4(1.0, 1.0, 1.0, 1.0), &characters, &mut char_quad);
                } else {
                    let turn = if game.host_turn { "You are playing." } else { "Client is playing." };
                    render_text(&text_shader, turn.to_string(),  800.0, 640.0, 0.5, vec4(1.0, 1.0, 1.0, 1.0), &characters, &mut char_quad);
                }
            }

            match game.chess.get_state() {
                ludviggl_chess::State::SelectPiece => {
                    if game.state == HostState::TestingClientMove {
                        let m = (game.recieved_move.0 as u8, game.recieved_move.1 as u8);
                        game.chess.select_piece(m.0, m.1).unwrap();
                        
                        if game.chess.get_state() as i8 == ludviggl_chess::State::SelectPiece as i8 {
                            game.state = HostState::BadClientMove;
                        }
                    }
                }

                ludviggl_chess::State::SelectMove => {
                    if game.state == HostState::TestingClientMove {
                        let p_player = game.chess.get_current_player() as i8;
                        let m = (game.recieved_move.2 as u8, game.recieved_move.3 as u8);
                        game.chess.select_move(m.0, m.1).unwrap();

                        if game.chess.get_state() as u8 == 3 {
                            game.state = HostState::PromotingClient;
                        } else if p_player == game.chess.get_current_player() as i8 {
                            game.state = HostState::BadClientMove;
                        } else {
                            game.state = HostState::RespondToClient;
                        }
                    }
                }

                _ => { }
            }

            match game.chess.get_state() {
                ludviggl_chess::State::CheckMate => {
                    game.game_end = true;
                }

                ludviggl_chess::State::SelectPromotion => {
                    game.chess.select_promotion(ludviggl_chess::Piece::Queen).unwrap();
                    if game.state == HostState::PromotingClient {
                        game.state = HostState::RespondToClient;
                    } else {
                        game.state = HostState::SendToClient;
                    }
                }

                _ => { }
            }

            for (_, event) in flush_messages(&events) {
                match event {
                    WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                        window.set_should_close(true);
                    }

                    WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                        if !game.game_end && !game.promoting && game.state == HostState::HostPlaying && game.client_connected {
                            host_on_pick(&mut game, &window);
                        }
                    }

                    _ => {}
                }

                if game.promoting && game.host_turn {
                    match event {
                        WindowEvent::Key(Key::Num1, _, Action::Press, _) => { game.selected_prom = ludviggl_chess::Piece::Rook; }
                        WindowEvent::Key(Key::Num2, _, Action::Press, _) => { game.selected_prom = ludviggl_chess::Piece::Knight; }
                        WindowEvent::Key(Key::Num3, _, Action::Press, _) => { game.selected_prom = ludviggl_chess::Piece::Bishop; }
                        WindowEvent::Key(Key::Num4, _, Action::Press, _) => { game.selected_prom = ludviggl_chess::Piece::Queen; }

                        _ => { }
                    }
                }
            }

            window.swap_buffers();
            glfw.poll_events();

            std::mem::drop(game);
            std::thread::sleep(std::time::Duration::from_millis(7));
        }
    }
}

fn host_on_pick(game: &mut HostGame, window: &Window) {
    let cursor = window.get_cursor_pos();
    
    if cursor.0 <= 800.0 { 
        let x: usize = (((cursor.0 as f32 * 8.0)) / 800.0).floor() as usize;
        let y: usize = 7 - (((cursor.1 as f32 * 8.0)) / 800.0).floor() as usize;

        for i in 0..64 {
            game.board[i].color = game.board[i].default_color;
        }

        match game.chess.get_state() {
            ludviggl_chess::State::SelectPiece => {
                game.chess.select_piece(x as u8, y as u8).unwrap();

                if game.chess.get_state() as i8 == ludviggl_chess::State::SelectPiece as i8 {
                } else {
                    game.made_move.0 = x;
                    game.made_move.1 = y;

                }

                if let Ok(moves) = game.chess.get_moves() {
                    game.board[8*y as usize + x as usize].color =
                        game.board[8*y as usize + x as usize].color +
                        Vec4{ x: 0.4, y: 0.4, z: 0.0, w: 0.0 };
                    
                    for &(a, b) in moves {
                        game.board[8*b as usize + a as usize].color =
                            game.board[8*b as usize + a as usize].color +
                            Vec4{ x: 0.0, y: 0.4, z: 0.0, w: 0.0 };
                    }
                }
            }

            ludviggl_chess::State::SelectMove => {
                let p_player = game.chess.get_current_player() as i8;
                game.chess.select_move(x as u8, y as u8).unwrap();

                if p_player == game.chess.get_current_player() as i8 {
                } else {
                    game.made_move.2 = x;
                    game.made_move.3 = y;
                    if game.chess.get_state() as i8 == 3 {
                        game.state = HostState::PromotingHost;
                    } else {
                        game.state = HostState::SendToClient;
                    }
                }
            }

            _ => { }
        }
    }
}

fn client_on_pick(game: &mut ClientGame, window: &Window) {
    let cursor = window.get_cursor_pos();
    
    if cursor.0 <= 800.0 { 
        let x: usize = (((cursor.0 as f32 * 8.0)) / 800.0).floor() as usize;
        let y: usize = 7 - (((cursor.1 as f32 * 8.0)) / 800.0).floor() as usize;

        if game.m_from == (usize::MAX, usize::MAX) {
            game.board[y*8 + x].color = vec4(1.0, 0.0, 0.0, 1.0);

            game.m_from = (x, y);
            return;
        }

        if game.m_to == (usize::MAX, usize::MAX) {
            for i in 0..64 {
                game.board[i].color = game.board[i].default_color;
            }

            game.m_to = (x, y);
            game.send_move = true;
            return;
        }
    }
}

fn render_text(shader: &Shader, text: String, x: f32, y: f32, scale: f32, color: Vec4, characters: &HashMap<char, Character>, char_quad: &mut Model2D) {
    shader.set_vec4("color", color);
    unsafe {
    gl::ActiveTexture(gl::TEXTURE0);
    gl::BindVertexArray(char_quad.vao);

    let mut x_: f32 = x;
    
    for c in text.chars() {
        let ch = characters.get(&c).unwrap();

        let xpos = x_ + ch.bearing.x as f32 * scale;
        let ypos = y - (ch.size.y - ch.bearing.y) as f32 * scale;

        let w = ch.size.x as f32 * scale;
        let h = ch.size.y as f32 * scale;
        let vertices: [[f32; 4]; 6] = [
            [ xpos,     ypos + h, 0.0, 0.0 ],            
            [ xpos,     ypos,     0.0, 1.0 ],
            [ xpos + w, ypos,     1.0, 1.0 ],
            [ xpos,     ypos + h, 0.0, 0.0 ],
            [ xpos + w, ypos,     1.0, 1.0 ],
            [ xpos + w, ypos + h, 1.0, 0.0 ]           
        ];

        gl::BindTexture(gl::TEXTURE_2D, ch.texture_id);
        gl::BindBuffer(gl::ARRAY_BUFFER, char_quad.vbo);
        gl::BufferSubData(gl::ARRAY_BUFFER, 0, std::mem::size_of::<f32>() as isize * 6 * 4, vertices.as_ptr().cast());
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        gl::DrawArrays(gl::TRIANGLES, 0, 6);

        x_ += (ch.advance >> 6) as f32 * scale;
    }
    }
}

fn load_ttf(characters: &mut HashMap<char, Character>, face: freetype::Face) {
    unsafe {
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    for c in 0..128u8 {
        if face.load_char(c as usize, LoadFlag::RENDER).is_err() {
            panic!("Failed to load glyph...");
        }
        
        let mut tex: u32 = 0;
        gl::GenTextures(1, &mut tex);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            face.glyph().bitmap().width(),
            face.glyph().bitmap().rows(),
            0,
            gl::RED,
            gl::UNSIGNED_BYTE,
            face.glyph().bitmap().buffer().as_ptr().cast()
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        let character = Character{
            texture_id: tex,
            size: IVec2{ x: face.glyph().bitmap().width(), y: face.glyph().bitmap().rows() },
            bearing: IVec2{ x: face.glyph().bitmap_left(), y: face.glyph().bitmap_top() },
            advance: face.glyph().advance().x as u32
        };

        characters.insert(c as char, character);
    }
    gl::BindTexture(gl::TEXTURE_2D, 0);
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
    }
}

fn load_texture(path: &str) -> u32 {
    let mut tex: u32 = 0;

    let mut f = std::fs::File::open(path).expect("File not found...");
    let mut contents = vec![];
    if f.read_to_end(&mut contents).is_err() {
        panic!("Failed to read file...");
    }

    let mut width: i32 = 0;
    let mut height: i32 = 0;
    let mut channels: i32 = 0;
    let image: *mut u8;

    unsafe {
    stb_image_rust::stbi_set_flip_vertically_on_load(1);
    image = stb_image_rust::stbi_load_from_memory(
        contents.as_mut_ptr(), 
        contents.len() as i32,
        &mut width,
        &mut height,
        &mut channels,
        stb_image_rust::STBI_rgb_alpha
    );

    gl::GenTextures(1, &mut tex);
    gl::BindTexture(gl::TEXTURE_2D, tex);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGBA as i32,
        width,
        height,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        image.cast()
    );
    gl::GenerateMipmap(gl::TEXTURE_2D);

    stb_image_rust::c_runtime::free(image);
    }

    return tex;
}

fn orthographic_projection(right: f32, left: f32, top: f32, bottom: f32, near: f32, far: f32) -> glm::Mat4 {
    let mut m = mat4(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    );
    m[0][0] = 2.0 / (right - left);
    m[1][1] = 2.0 / (top - bottom);
    m[2][2] = 1.0 / (far - near);
    m[3][0] = -(right + left) / (right - left);
    m[3][1] = -(top + bottom) / (top - bottom);
    m[3][2] = -near / (far - near);

    return m;
}