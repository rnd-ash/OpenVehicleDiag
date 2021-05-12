
use std::cmp::min;

use image::{GenericImageView, ImageFormat};
use std::io::Write;
use std::io::Read;
use std::io::{self, BufReader};

mod commapi;
mod passthru;

use crate::{
    commapi::{comm_api::ComServer, passthru_api::PassthruApi, protocols::kwp2000::KWP2000ECU},
    commapi::{comm_api::ISO15765Config, protocols::ProtocolServer},
    passthru::{PassthruDevice, PassthruDrv},
};

#[derive(Debug, Copy, Clone)]
struct Line {
    start_x: u8,
    start_y: u8,
    end_x: u8,
    end_y: u8,
}

fn main() {

    let args: Vec<String> = std::env::args().collect();

    const LCD_WIDTH: u32 = 60;
    const LCD_HEIGHT: u32 = 100;

    let dev = PassthruDevice::find_all().expect("Couldn't find any passthru adapters for test")
        [0]
    .clone();
    let drv = PassthruDrv::load_lib(dev.drv_path.clone()).expect("Couldn't load library");

    // Open a Comm server link with my custom Passthru adapter
    let mut api = PassthruApi::new(dev, drv).clone_box();
    api.open_device().expect("Could not open device!");

    // Start ISO-TP KWP2000 session with IC
    let mut server = KWP2000ECU::start_diag_session(
        api,
        &ISO15765Config {
            baud: 83_333,
            send_id: 1460,
            recv_id: 1268,
            block_size: 8,
            sep_time: 20,
            use_ext_isotp: false,
            use_ext_can: false
        },
        Some(0x001C),
    )
    .expect("Error opening connection with IC ECU");

    let input_file = std::fs::File::open(args[1].clone()).unwrap();
    let mut output_file = std::fs::File::create(args[2].clone()).unwrap();

    // W203 IC is 56 pixels wide, ~100 tall for the top zone
    let img = image::load(BufReader::new(input_file), ImageFormat::Png)
        .expect("Error loading image");

    // get scale bounds for the image
    let sf = (img.width() as f32 / LCD_WIDTH as f32) as f32;

    let mut lines: Vec<Line> = Vec::new();
    // Drawing a large vertical line seems to clear the LCD in test mode

    let mut lines2: Vec<Line> = Vec::new();
    // Drawing a large vertical line seems to clear the LCD in test mode

    let max_height = std::cmp::min((img.height() as f32 / sf) as u32, LCD_HEIGHT);

    let y_offset: u8 = 60;

    for y in 0..max_height {
        let mut new_line = true;
        for x in 0..LCD_WIDTH {
            let x_coord = min((x as f32 * sf) as u32, img.width() - 1);
            let y_coord = min((y as f32 * sf) as u32, img.height() - 1);
            let px_colour = img.get_pixel(x_coord, y_coord);
            let rgb = px_colour.0;
            if rgb[0] > 128 || rgb[1] > 128 || rgb[2] > 128 {
                if new_line {
                    // Create new line
                    lines.push(Line {
                        start_x: x as u8,
                        start_y: y as u8 + y_offset,
                        end_x: x as u8,
                        end_y: y as u8 + y_offset,
                    });
                    new_line = false;
                } else {
                    // Append to last line in the matrix
                    if let Some(line) = lines.last_mut() {
                        line.end_x = x as u8;
                        line.end_y = y as u8 + y_offset;
                    }
                }
            } else {
                new_line = true; // We need to draw a new line
            }
        }
    }

    for x in 0..LCD_WIDTH {
        let mut new_line = true;
        for y in 0..max_height {
            let x_coord = min((x as f32 * sf) as u32, img.width() - 1);
            let y_coord = min((y as f32 * sf) as u32, img.height() - 1);
            let px_colour = img.get_pixel(x_coord, y_coord);
            let rgb = px_colour.0;
            if rgb[0] > 128 || rgb[1] > 128 || rgb[2] > 128 {
                if new_line {
                    // Create new line
                    lines2.push(Line {
                        start_x: x as u8,
                        start_y: y as u8 + y_offset,
                        end_x: x as u8,
                        end_y: y as u8 + y_offset,
                    });
                    new_line = false;
                } else {
                    // Append to last line in the matrix
                    if let Some(line) = lines2.last_mut() {
                        line.end_x = x as u8;
                        line.end_y = y as u8 + y_offset;
                    }
                }
            } else {
                new_line = true; // We need to draw a new line
            }
        }
    }

    let lines_ref = if lines.len() > lines2.len() {
        &lines2
    } else {
        &lines
    };

    let mut  pic_idx: u16 = 0;

    server.set_p2_timeout_interval(150);

    for l in lines_ref {
        // Send draw line command to LCD
        server.run_command(0x31, &[0x03, 0x06, l.start_x, l.start_y, l.end_x, l.end_y]);
    }

    // Get time-lapse photo from phone camera
    std::thread::spawn(move || {
        use curl::easy::Easy;
        use std::sync::Arc;

        let mut handle = Easy::new();
        handle.url("http://192.168.1.204:8080/ptz?zoom=15").unwrap();

        if let Err(e) = handle.perform() {
            eprintln!("Error setting Zoom level!");
            std::process::exit(1)
        }

        handle.url("http://192.168.1.204:8080/focus_distance?set=2.31").unwrap();

        if let Err(e) = handle.perform() {
            eprintln!("Error setting focus level!");
            std::process::exit(1)
        }

        let mut data: Vec<u8> = Vec::new();
        {
            let mut handle2 = Easy::new();
            handle2.url("http://192.168.1.204:8080/photo.jpg").unwrap();
            let mut transfer = handle2.transfer();
            transfer.write_function(|bytes| {
                data.extend_from_slice(bytes);
                Ok(bytes.len())
            }).unwrap();

            if let Err(e) = transfer.perform() {
                eprintln!("Error getting photo! {}", e);
                std::process::exit(2)
            }
        }

        // Write to file
        if let Err(e) = output_file.write_all(&data) {
            eprintln!("Error writing file!");
            std::process::exit(3)
        }

        // We are done. Exit thread!
        std::process::exit(0)
    });

    loop {
        server.run_command(0x31, &[03, 06, 00, 00, 00, 00]); // Keep the test active (Stops LCD from clearing after test)
    }
}
