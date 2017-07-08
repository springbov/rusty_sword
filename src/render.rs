extern crate termion;

use termion::raw::*;
use termion::clear;
use termion::color;
use termion::input::MouseTerminal;

use actor::*;
use floor::*;
use primitive::*;

use std::io::{Write, stdout};
use std::sync::*;
use std::thread;
use std::time::Duration;

fn render_floor<W : Write>(screen : &mut W, floor: &Floor) {
    for row in &floor.tiles {
        let mut line = String::new();
        for tile in row {
            if let Some(ref wall) = tile.wall {
                line.push_str(wall);
            } else {
                line.push_str(" "); // U-2027
                //line.push_str("‧"); // U-2027
            }
        }
        write!(screen, "{}\n\r", line).unwrap(); // U-2502
    }
}

fn goto_cursor_coord(coord : &Coord) -> termion::cursor::Goto {
    // Coordinate translation naively assumes floor is being rendered at 1,1
    termion::cursor::Goto(coord.col+1, coord.row+1)
}


pub fn render_loop(floor        : Arc<Mutex<Floor>>,
                   dirty_coords : Arc<Mutex<Vec<Coord>>>,
                   messages     : Arc<Mutex<Vec<String>>>,
                   player       : Arc<Mutex<Player>>,
                   monsters     : Arc<Mutex<Vec<Monster>>>,
                   stop         : Arc<Mutex<bool>>) {

    let mut screen = MouseTerminal::from(stdout().into_raw_mode().unwrap());
    // Hide the cursor, clear the screen
    write!(screen, "{}{}", termion::cursor::Hide, termion::clear::All).unwrap();

    {
        write!(screen, "{}", termion::cursor::Goto(1, 1)).unwrap();
        let floor = floor.lock().unwrap();
        render_floor(&mut screen, &floor);
    }

    // Render Loop
    loop {
        // Don't render too hot.
        thread::sleep(Duration::from_millis(10));

        // Time to stop?
        {
            if *stop.lock().unwrap() {
                break;
            }
        }

        // Once we can lock floor, we can lock anything else we want in this thread.
        let mut floor = floor.lock().unwrap();

        // Redraw any dirty coordinates
        let mut dirty_coords = dirty_coords.lock().unwrap();
        for coord in dirty_coords.drain(..) {
            write!(screen, "{}{}", goto_cursor_coord(&coord), floor.get_symbol(&coord)).unwrap();
        }

        // Render Player
        {
            let mut player = player.lock().unwrap();
            if player.dirty {
                player.dirty = false;
                // Player's sword
                write!(screen, "{}", goto_cursor_coord(&player.sword_coord)).unwrap();
                write!(screen, "{}", &sword_symbol(&player.facing)).unwrap();
                // Player himself
                write!(screen, "{}", goto_cursor_coord(&player.coord)).unwrap();
                write!(screen, "{}", &player.symbol).unwrap();
            }
        }

        // Render Monsters
        {
            let monsters = monsters.lock().unwrap();
            for monster in monsters.iter() {
                write!(screen, "{}", goto_cursor_coord(&monster.coord)).unwrap();
                write!(screen, "{}", &monster.symbol).unwrap();
            }
        }

        // Bottom text
        write!(screen, "{}", termion::cursor::Goto(1, (floor.rows+1) as u16)).unwrap();
        write!(screen, "{}\n\r\n\r", floor.name).unwrap(); // Dungeon Name
        let messages = messages.lock().unwrap();
        for msg in messages.iter() {
            write!(screen, "{}{}", color::Fg(color::LightWhite), msg).unwrap();
            write!(screen, "{}{}\n\r", color::Fg(color::Reset), clear::UntilNewline).unwrap();
        }

        screen.flush().unwrap();
    }

    // Nice cleanup: Move cursor below the floor, so we can see how we finished
    {
        let floor = floor.lock().unwrap();
        write!(screen, "{}", goto_cursor_coord(&Coord { col: 0, row: (floor.rows+7) as u16})).unwrap();
    }
    print!("{}", termion::cursor::Show);
    screen.flush().unwrap();
}

