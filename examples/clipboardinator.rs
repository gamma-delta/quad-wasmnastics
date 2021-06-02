#![feature(drain_filter)]
#![feature(try_blocks)]

use macroquad::prelude::*;
use quad_wasmnastics::{clipboard, console_log, storage};

#[macroquad::main("Clipboardinator")]
async fn main() {
    let strings: anyhow::Result<Vec<String>> = try {
        let load_strings = storage::load()?;
        serde_json::from_slice(&load_strings)?
    };
    let mut strings = match strings {
        Ok(it) => it,
        Err(oh_no) => {
            console_log!("{:?}", oh_no);
            vec![String::from("try me!")]
        }
    };

    let mut copy_queue = Vec::new();
    let mut paste_queue = Vec::new();

    loop {
        let mut save = false;
        if is_key_pressed(KeyCode::V) {
            paste_queue.push(clipboard::get_clipboard());
        }
        if is_key_pressed(KeyCode::C) && !strings.is_empty() {
            copy_queue.push(clipboard::set_clipboard(strings.remove(0)));
            save = true;
        }

        // well if you're hacking on this crate you need nightly rust anyways
        // drain filter go brr
        copy_queue
            // remove it if it's some
            .drain_filter(|waiter| {
                if waiter.try_get().is_some() {
                    save = true;
                    // kill this
                    true
                } else {
                    false
                }
            })
            .for_each(drop);
        // we don't actually need to keep a list of copies around
        // oh well
        // hindsight is 2020

        paste_queue
            .drain_filter(|waiter| {
                if let Some(copied) = waiter.try_get() {
                    strings.push(copied);
                    save = true;
                    true
                } else {
                    if is_key_pressed(KeyCode::Q) {
                        info!("{:?}", &waiter);
                    }
                    false
                }
            })
            .for_each(drop);

        if save {
            let res: anyhow::Result<()> = try {
                let save_strings = serde_json::to_string(&strings)?;
                storage::save(save_strings)?;
            };
            if let Err(oh_no) = res {
                console_log!("{:?}", oh_no);
            }
        }

        clear_background(WHITE);

        draw_text("CLIPBOARDINATOR", 20.0, 20.0, 32.0, BLACK);
        draw_text(
            "Press V to push your clipboard contents.",
            20.0,
            40.0,
            32.0,
            BLACK,
        );
        draw_text(
            "Press C to dequeue into your clipboard.",
            20.0,
            60.0,
            32.0,
            BLACK,
        );
        draw_text(
            "Your data is saved into localstorage!",
            20.0,
            80.0,
            32.0,
            BLACK,
        );

        for (idx, string) in strings.iter().enumerate() {
            let y = 120.0 + idx as f32 * 20.0;
            draw_text(&format!("{}. {}", idx + 1, string), 20.0, y, 32.0, PURPLE);
        }

        next_frame().await
    }
}
