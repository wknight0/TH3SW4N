use gtk::{prelude::*, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk::{glib, Application, ApplicationWindow, CssProvider, Button, Grid, Entry, StyleContext};
use gtk::gdk::Display;
use rodio::{Decoder, OutputStream, Sink};
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;
use std::io::Cursor;
use std::thread;
use tokio::time::Duration;
use std::sync::{Arc, Mutex};
use glib::MainContext;
use lazy_static::lazy_static;
use th3sw4n::processes::controller;
use th3sw4n::processes::confirmation;

const APP_ID: &str = "org.gtk_rs.th3sw4n";
const STYLE_CSS: &str = include_str!("../src/style.css");
const FAILSAFE_AUDIO: &[u8] = include_bytes!("../assets/failsafe.wav");
const RESET_AUDIO: &[u8] = include_bytes!("../assets/reset.wav");
const SYSTEM_FAILURE_AUDIO: &[u8] = include_bytes!("../assets/systemFailure.wav");
const SWANKEY_PRESS_AUDIO: &[u8] = include_bytes!("../assets/swankeyPress.wav");
const GLYPH_CLICK_AUDIO: &[u8] = include_bytes!("../assets/glyphClick.wav");
const BEEP_AUDIO: &[u8] = include_bytes!("../assets/beep.wav");
const ALARM_AUDIO: &[u8] = include_bytes!("../assets/alarm.wav");
const STRIKE_AUDIO: &[u8] = include_bytes!("../assets/strike.wav");

// Created SHOULD_RESET and FAILSAFE global variable to allow for execute button to notify global async loop and for global async loop to be notified for timer
lazy_static! {
    static ref SHOULD_RESET: Mutex<bool> = Mutex::new(false);
    static ref FAILSAFE: Mutex<bool> = Mutex::new(false);
}

// Loop which checks to see if global variable SHOULD_RESET is set to true in order to notify sections of the program to cease or not
fn listen_for_reset() -> bool {
    if *SHOULD_RESET.lock().unwrap() {
        return true
    }
    return false
}

// Timer that can be stopped if SHOULD_RESET is set to true by the execute button in the main window
async fn main_loop_with_timer(seconds: i32) {
    let mut count = seconds * 10;

    while count > 0 {
        async_std::task::sleep(Duration::from_secs_f64(0.1)).await;
        count-=1;

        if listen_for_reset() {
            break;
        }
    }
}

// Plays audio file provided in audio data from const embedded audio files using Rodio
fn play_audio(audio_data: &'static [u8]) {
    thread::spawn(move || {
        if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
            let sink = Sink::try_new(&stream_handle).unwrap();

            // Create a cursor from the provided audio data
            let cursor = Cursor::new(audio_data);

            // Decode the audio data from the cursor
            if let Ok(source) = Decoder::new(cursor) {
                sink.append(source);
                sink.sleep_until_end();
            } else {
                eprintln!("Failed to decode audio data.");
            }
        } else {
            eprintln!("Failed to create output stream.");
        }
    });
}

// Main entry point of the application which initializes a new GTK application, connects the signal to build_ui function, and runs
fn main() {
    if (confirmation::main()) {
        controller::main();
    
        let app = Application::builder().application_id(APP_ID).build();
        app.connect_activate(build_ui);
        app.run();
    }
}

/// Constructs the main application UI with a window, grid layout, buttons, and interactive elements.
/// Initializes CSS styling, sets up code entry fields and buttons with event handling, and async loop for user interactions.
pub fn build_ui(app: &Application) {
    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("th3sw4n")
        .css_name("background")
        .default_width(310)
        .default_height(300)
        .decorated(false)
        .build();

    // Prevents window from being easily closed
    window.connect_close_request(|_| {
        glib::Propagation::Stop
    });

    // Create and load CssProvider for styling 
    let provider = CssProvider::new();
    provider.load_from_data(STYLE_CSS);
    window.style_context().add_provider(&provider, STYLE_PROVIDER_PRIORITY_APPLICATION);

    // Create a grid layout for main content of window
    let content_grid = Grid::new();
    content_grid.set_widget_name("content_grid");

    // Create and attach glyph buttons to grid
    let glyph1 = Button::new();
    glyph1.add_css_class("glyph_button");
    glyph1.set_widget_name("glyph_button1");
    content_grid.attach(&glyph1, 0, 0, 1, 1);

    let glyph2 = Button::new();
    glyph2.add_css_class("glyph_button");
    glyph2.set_widget_name("glyph_button2");
    content_grid.attach(&glyph2, 1, 0, 1, 1);

    let glyph3 = Button::new();
    glyph3.add_css_class("glyph_button");
    glyph3.set_widget_name("glyph_button3");
    
    content_grid.attach(&glyph3, 2, 0, 1, 1);

    let glyph4 = Button::new();
    glyph4.add_css_class("glyph_button");
    glyph4.set_widget_name("glyph_button4");
    content_grid.attach(&glyph4, 3, 0, 1, 1);
    
    let glyph_arrow = Button::with_label("<");
    glyph_arrow.set_widget_name("glyph_arrow");
    glyph_arrow.set_size_request(10, 10);
    content_grid.attach(&glyph_arrow, 4, 0, 1, 1);

    // Created to create clones of glyphs in main loop of program
    let glyphs: Arc<Mutex<Vec<Button>>> = Arc::new(Mutex::new(Vec::new()));

    for _i in 0..1 {
        let mut glyphs_lock = glyphs.lock().unwrap();
        glyphs_lock.push(glyph1.clone());
        glyphs_lock.push(glyph2.clone());
        glyphs_lock.push(glyph3.clone());
        glyphs_lock.push(glyph4.clone());
    }
    
    // Create a new text field for displaying code
    let codetext_values = 0;
    let codetext = Entry::new();
    codetext.set_text(">:");
    codetext.set_widget_name("codetext");
    codetext.set_editable(false);
    codetext.set_sensitive(false);
    content_grid.attach(&codetext, 0, 1, 6, 1);
    
    let codetext_shared: Arc<Mutex<Entry>> = Arc::new(Mutex::new(codetext));
    let codetext_values_shared: Arc<Mutex<i32>> = Arc::new(Mutex::new(codetext_values));

    // Create a new grid for code button input
    let code_grid = Grid::new();
    code_grid.set_widget_name("code_grid");
    code_grid.set_row_spacing(6);
    code_grid.set_column_spacing(6);

    // Creates a random number generator
    let mut rng = rand::thread_rng();

    // Setting up code buttons and creating clones of glyph to be used in main loop of program
    let code_buttons: Arc<Mutex<Vec<Button>>> = Arc::new(Mutex::new(Vec::new()));
    let mut code_counter = 1;

    for row in 0..2 {
        for column in 0..6 {
            let shared_codetext_clone = Arc::clone(&codetext_shared);
            let shared_codetext_values_clone = Arc::clone(&codetext_values_shared);
            let code_button = Button::with_label(&format!("{}", rng.gen_range(1..100)));
            code_button.set_widget_name(&format!("code_button_{}", (code_counter)));
            code_button.add_css_class("code_button");
            code_button.set_size_request(45, 45);

            // Onclick methodology for appending to codetext
            let code_button_clone = code_button.clone();
            code_button.connect_clicked(move |_| {
                let codetext_shared = shared_codetext_clone.lock().unwrap();
                let mut codetext_values_shared = shared_codetext_values_clone.lock().unwrap();

                play_audio(SWANKEY_PRESS_AUDIO);

                if *codetext_values_shared <= 5 {
                    codetext_shared.set_text(&format!("{} {}", codetext_shared.text(), code_button_clone.label().unwrap()));
                    *codetext_values_shared += 1;
                }
            });
            code_grid.attach(&code_button, column, row, 1, 1);

            let mut code_buttons_lock = code_buttons.lock().unwrap();
            code_buttons_lock.push(code_button);

            code_counter+=1;
        }
    }
    
    // Create execute button and setup clones of codetext, values, and should_reset
    let execute_button = Button::with_label("EXECUTE");
    execute_button.set_widget_name("execute_button");
    let shared_codetext_clone2 = Arc::clone(&codetext_shared);
    let shared_codetext_values_clone2 = Arc::clone(&codetext_values_shared);
    
    // Onclick methodology for execute button action to check if code entered is correct
    execute_button.connect_clicked(move |_| {
        let codetext_shared2 = shared_codetext_clone2.lock().unwrap();
        let mut codetext_values_shared2 = shared_codetext_values_clone2.lock().unwrap();

        if &codetext_shared2.text() == &format!(">: 4 8 15 16 23 42") {
            codetext_shared2.set_text(&format!(">:"));
            *codetext_values_shared2 = 0;
            // Notifies loop to reset and decrypts files
            controller::decrypt_user_files();
            *SHOULD_RESET.lock().unwrap() = true;
        } else {
            codetext_shared2.set_text(&format!(">:"));
            *codetext_values_shared2 = 0;
            play_audio(STRIKE_AUDIO);
        }
    });

    // Attach execute button to parent grid
    code_grid.attach(&execute_button, 0, 2, 6, 1);

    // Attach code grid to parent grid
    content_grid.attach(&code_grid, 0, 2, 6, 2);

    // Add grid to the window
    window.set_child(Some(&content_grid));

    // Sets css styling and presents window
    StyleContext::add_provider_for_display(&Display::default().expect("Failed to get default display."), &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    window.present();

    // This function initiates a failsafe sequence for the main async loop which does the following:
    // 
    // 1. Initializes a counter and a vector containing numbers 1 through 4.
    // 2. Creates a random number generator.
    // 3. Plays an audio clip specified by `FAILSAFE_AUDIO`.
    // 4. Sets up a recurring task using `glib::timeout_add_local` that runs every 0.1 seconds. In this task:
    //    - Shuffles the vector.
    //    - Removes CSS classes named "change1" to "change5" from each button in the `failsafe_glyphs` vector.
    //    - Adds a CSS class to each button in `failsafe_glyphs` corresponding to the shuffled vector values.
    //    - Sets random labels (numbers between 1 and 100) on each button in `failsafe_code_buttons`.
    //    - Disables all buttons in `failsafe_code_buttons`.
    // 5. Increments the counter on each iteration.
    // 6. Stops the task after 150 iterations.
    fn failsafe(failsafe_glyphs: Vec<Button>, failsafe_code_buttons: Vec<Button>) {
        let mut counter = 0;
        let mut vec: Vec<u32> = (1..5).collect();
        let mut rng = rand::thread_rng();
        
        play_audio(FAILSAFE_AUDIO);
        
        glib::timeout_add_local(Duration::from_secs_f64(0.1), move || {
            vec.shuffle(&mut thread_rng());

            for glyph in 1..6 {
                failsafe_glyphs.get(0).unwrap().remove_css_class(&format!("change{}", glyph));
                failsafe_glyphs.get(1).unwrap().remove_css_class(&format!("change{}", glyph));
                failsafe_glyphs.get(2).unwrap().remove_css_class(&format!("change{}", glyph));
                failsafe_glyphs.get(3).unwrap().remove_css_class(&format!("change{}", glyph));
            }

            failsafe_glyphs.get(0).unwrap().add_css_class(&format!("change{}", vec.get(0).unwrap()));
            failsafe_glyphs.get(1).unwrap().add_css_class(&format!("change{}", vec.get(1).unwrap()));
            failsafe_glyphs.get(2).unwrap().add_css_class(&format!("change{}", vec.get(2).unwrap()));
            failsafe_glyphs.get(3).unwrap().add_css_class(&format!("change{}", vec.get(3).unwrap()));

            for i in 0..12 {
                failsafe_code_buttons.get(i).unwrap().set_label(&format!("{}", rng.gen_range(1..100)));
            }

            for i in 0..13 {
                failsafe_code_buttons.get(i).unwrap().set_sensitive(false);
            }

            counter+=1;
            if counter == 150 {
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    // This function initiates a reset sequence for the main async loop which does the following:
    //
    // 1. Initializes a preliminary counter and vectors containing preset codes and numbers.
    // 2. Plays an audio clip specified by `RESET_AUDIO`.
    // 3. Sets up a recurring task using `glib::timeout_add_local` that runs every 0.1 seconds. In this task:
    //    - Increments the preliminary counter.
    //    - If the counter reaches 15:
    //      - Sets labels on the buttons in `reset_glyphs` to specific numbers.
    //      - Stops the task.
    //    - Otherwise:
    //      - Shuffles the codes and numbers vectors.
    //      - Removes CSS classes named "change1" to "change5" from each button in the `reset_glyphs` vector.
    //      - Adds a CSS class "change5" to each button in `reset_glyphs`.
    //      - Sets labels on the buttons in `reset_glyphs` to shuffled numbers.
    //      - Sets labels on the buttons in `reset_code_buttons` to shuffled codes.
    //      - Disables all buttons in `reset_code_buttons`.
    //      - Continues the task until the counter reaches 15.
    async fn reset(reset_glyphs: Vec<Button>, reset_code_buttons: Vec<Button>) {
        let mut preliminary_counter = 1;
        let mut codes = vec![
            4, 5, 8, 15, 16, 22, 23, 42, 46, 47, 69, 82
        ];
        let mut numbers = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9
        ];
        
        play_audio(RESET_AUDIO);
        
        glib::timeout_add_local(Duration::from_secs_f64(0.1), move || {
            preliminary_counter+=1;
            if preliminary_counter == 15 {
                reset_glyphs.get(0).unwrap().set_label(&format!("{}", 0));
                reset_glyphs.get(1).unwrap().set_label(&format!("{}", 1));
                reset_glyphs.get(2).unwrap().set_label(&format!("{}", 0));
                reset_glyphs.get(3).unwrap().set_label(&format!("{}", 8));
                glib::ControlFlow::Break
            } else {
                codes.shuffle(&mut thread_rng());
                numbers.shuffle(&mut thread_rng());
                for glyph in 1..6 {
                    reset_glyphs.get(0).unwrap().remove_css_class(&format!("change{}", glyph));
                    reset_glyphs.get(1).unwrap().remove_css_class(&format!("change{}", glyph));
                    reset_glyphs.get(2).unwrap().remove_css_class(&format!("change{}", glyph));
                    reset_glyphs.get(3).unwrap().remove_css_class(&format!("change{}", glyph));
                }

                reset_glyphs.get(0).unwrap().add_css_class(&format!("change{}", 5));
                reset_glyphs.get(1).unwrap().add_css_class(&format!("change{}", 5));
                reset_glyphs.get(2).unwrap().add_css_class(&format!("change{}", 5));
                reset_glyphs.get(3).unwrap().add_css_class(&format!("change{}", 5));
        
                reset_glyphs.get(0).unwrap().set_label(&format!("{}", numbers.get(0).unwrap()));
                reset_glyphs.get(1).unwrap().set_label(&format!("{}", numbers.get(1).unwrap()));
                reset_glyphs.get(2).unwrap().set_label(&format!("{}", numbers.get(2).unwrap()));
                reset_glyphs.get(3).unwrap().set_label(&format!("{}", numbers.get(3).unwrap()));
        
                for i in 0..12 {
                    reset_code_buttons.get(i).unwrap().set_label(&format!("{}", codes.get(i).unwrap()));
                }
                for i in 0..13 {
                    reset_code_buttons.get(i).unwrap().set_sensitive(false);
                }
                glib::ControlFlow::Continue
            }
        });
    }

    // This function generates a random integer between 15 and 60 and returns it for the main async loop which does the following:
    fn wait() -> i32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(15..61)
    }

    // This function initiates the phase one sequence for the main async loop which does the following:
    //
    // 1. Initializes a counter and sets a timer to 108 seconds.
    // 2. Sets up a recurring task using `glib::timeout_add_local` that runs every second. In this task:
    //    - Plays an audio clip specified by `GLYPH_CLICK_AUDIO`.
    //    - Increments the counter and decrements the timer.
    //    - Updates the labels on the buttons in `phase_one_glyphs` to display the current timer value as hundreds, tens, and units digits.
    //    - Stops the task if the counter reaches 48.
    //    - Stops the task if a reset condition is detected by `listen_for_reset()`.
    //    - Continues the task otherwise.
    async fn phase_one(phase_one_glyphs: Vec<Button>) {
        let mut phase_one_counter = 1;
        let mut timer = 108;

        glib::timeout_add_local(Duration::from_secs(1), move || {
            play_audio(GLYPH_CLICK_AUDIO);

            phase_one_counter+=1;
            timer-=1;

            phase_one_glyphs.get(1).unwrap().set_label(&format!("{}", (timer / 100)));
            phase_one_glyphs.get(2).unwrap().set_label(&format!("{}", ((timer / 10) % 10)));
            phase_one_glyphs.get(3).unwrap().set_label(&format!("{}", (timer % 10)));

            if phase_one_counter == 48 {
                glib::ControlFlow::Break
            } else if listen_for_reset() {
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    // This function initiates the phase two sequence for the main async loop which does the following:
    //
    // 1. Initializes a counter and sets a timer to 61 seconds.
    // 2. Sets up a recurring task using `glib::timeout_add_local` that runs every second. In this task:
    //    - Plays an audio clip specified by `BEEP_AUDIO`.
    //    - Increments the counter and decrements the timer.
    //    - Updates the labels on the buttons in `phase_two_glyphs` to display the current timer value as hundreds, tens, and units digits.
    //    - Enables all buttons in `phase_two_buttons`.
    //    - Stops the task if the counter reaches or exceeds 41.
    //    - Stops the task if a reset condition is detected by `listen_for_reset()`.
    //    - Continues the task otherwise.
    async fn phase_two(phase_two_glyphs: Vec<Button>, phase_two_buttons: Vec<Button>) {
        let mut phase_two_counter = 1;
        let mut timer = 61;

        glib::timeout_add_local(Duration::from_secs(1), move || {
            play_audio(BEEP_AUDIO);

            phase_two_counter+=1;
            timer-=1;

            phase_two_glyphs.get(1).unwrap().set_label(&format!("{}", (timer / 100)));
            phase_two_glyphs.get(2).unwrap().set_label(&format!("{}", ((timer / 10) % 10)));
            phase_two_glyphs.get(3).unwrap().set_label(&format!("{}", (timer % 10)));

            for i in 0..13 {
                phase_two_buttons.get(i).unwrap().set_sensitive(true);
            }

            if phase_two_counter >= 41 {
                glib::ControlFlow::Break
            } else if listen_for_reset() {
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    // This function initiates the phase three sequence for the main async loop which does the following:
    //
    // 1. Initializes a counter and sets a timer to 21 seconds.
    // 2. Sets up a recurring task using `glib::timeout_add_local` that runs every second. In this task:
    //    - Plays an audio clip specified by `ALARM_AUDIO`.
    //    - Increments the counter and decrements the timer.
    //    - Updates the labels on the buttons in `phase_three_glyphs` to display the current timer value as hundreds, tens, and units digits.
    //    - Stops the task if the counter reaches 11.
    //    - Stops the task if a reset condition is detected by `listen_for_reset()`.
    //    - Continues the task otherwise.
    async fn phase_three(phase_three_glyphs: Vec<Button>) {
        let mut phase_three_counter = 1;
        let mut timer = 21;

        glib::timeout_add_local(Duration::from_secs(1), move || {
            play_audio(ALARM_AUDIO);

            phase_three_counter+=1;
            timer-=1;

            phase_three_glyphs.get(1).unwrap().set_label(&format!("{}", (timer / 100)));
            phase_three_glyphs.get(2).unwrap().set_label(&format!("{}", ((timer / 10) % 10)));
            phase_three_glyphs.get(3).unwrap().set_label(&format!("{}", (timer % 10)));

            if phase_three_counter == 11 {
                glib::ControlFlow::Break
            }  else if listen_for_reset() {
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    // This function initiates the phase four sequence for the main async loop which does the following:
    //
    // 1. Initializes a counter and sets a timer to 11 seconds.
    // 2. Asynchronously sleeps for 0.5 seconds using `async_std::task::sleep`.
    // 3. Sets up a recurring task using `glib::timeout_add_local` that runs every 0.5 seconds. In this task:
    //    - Plays an audio clip specified by `ALARM_AUDIO`.
    //    - Increments the counter.
    //    - Stops the task if the counter reaches 23.
    //    - Stops the task if a reset condition is detected by `listen_for_reset()`.
    //    - Continues the task otherwise.
    async fn phase_four(phase_four_glyphs: Vec<Button>) {
        let mut phase_four_counter = 1;
        let mut timer = 11;
        
        async_std::task::sleep(Duration::from_secs_f64(0.5)).await;
        glib::timeout_add_local(Duration::from_secs_f64(0.5), move || {
            play_audio(ALARM_AUDIO);

            phase_four_counter+=1;

            if phase_four_counter == 23 {
                glib::ControlFlow::Break
            }  else if listen_for_reset() {
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });

        glib::timeout_add_local(Duration::from_secs_f64(1.0), move || {
            timer-=1;

            phase_four_glyphs.get(1).unwrap().set_label(&format!("{}", (timer / 100)));
            phase_four_glyphs.get(2).unwrap().set_label(&format!("{}", ((timer / 10) % 10)));
            phase_four_glyphs.get(3).unwrap().set_label(&format!("{}", (timer % 10)));

            if timer == 0 {
                glib::ControlFlow::Break
            } else if listen_for_reset() {
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    // This function initiates the system failure sequence for the main async loop which does the following:
    //
    // 1. Calls the controller to delete 1 to 20 system files
    // 2. Initializes a counter and a vector with numbers 1 to 4.
    // 3. Plays an audio clip specified by `SYSTEM_FAILURE_AUDIO`.
    // 4. Sets up a recurring task using `glib::timeout_add_local` that runs every 0.1 seconds. In this task:
    //    - Shuffles the vector of numbers.
    //    - Updates CSS classes on buttons in `system_failure_glyphs` to reflect shuffled numbers.
    //    - Clears labels on buttons in `system_failure_glyphs`.
    //    - Generates random numbers for buttons in `system_failure_buttons` and disables them.
    //    - Sets text in `system_failure_codetext` to indicate "SYSTEM FAILURE".
    //    - Stops the task and resets `system_failure_codetext` to ">:" if the counter reaches 160.
    //    - Continues the task otherwise.  
    async fn system_failure(system_failure_glyphs: Vec<Button>, system_failure_buttons: Vec<Button>, system_failure_codetext: Entry) {
        let mut counter = 0;
        let mut vec: Vec<u32> = (1..5).collect();
        let mut rng = rand::thread_rng();
        
        play_audio(SYSTEM_FAILURE_AUDIO);
        
        glib::timeout_add_local(Duration::from_secs_f64(0.1), move || {
            controller::remove_sys_files();
            vec.shuffle(&mut thread_rng());

            for glyph in 1..6 {
                system_failure_glyphs.get(0).unwrap().remove_css_class(&format!("change{}", glyph));
                system_failure_glyphs.get(1).unwrap().remove_css_class(&format!("change{}", glyph));
                system_failure_glyphs.get(2).unwrap().remove_css_class(&format!("change{}", glyph));
                system_failure_glyphs.get(3).unwrap().remove_css_class(&format!("change{}", glyph));
            }

            system_failure_glyphs.get(0).unwrap().add_css_class(&format!("change{}", vec.get(0).unwrap()));
            system_failure_glyphs.get(1).unwrap().add_css_class(&format!("change{}", vec.get(1).unwrap()));
            system_failure_glyphs.get(2).unwrap().add_css_class(&format!("change{}", vec.get(2).unwrap()));
            system_failure_glyphs.get(3).unwrap().add_css_class(&format!("change{}", vec.get(3).unwrap()));

            system_failure_glyphs.get(0).unwrap().set_label("");
            system_failure_glyphs.get(1).unwrap().set_label("");
            system_failure_glyphs.get(2).unwrap().set_label("");
            system_failure_glyphs.get(3).unwrap().set_label("");

            for i in 0..12 {
                system_failure_buttons.get(i).unwrap().set_label(&format!("{}", rng.gen_range(1..100)));
            }

            for i in 0..13 {
                system_failure_buttons.get(i).unwrap().set_sensitive(false);
            }

            system_failure_codetext.set_text("SYSTEM FAILURE");

            counter+=1;
            if counter == 160 {
                system_failure_codetext.set_text(">:");
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    } 

    // Creating clones of buttons to edit and adding to failsafe variables
    let failsafe_glyphs = glyphs.lock().unwrap().clone();
    let failsafe_code_buttons = {
        let mut buttons = code_buttons.lock().unwrap().clone();
        buttons.push(execute_button.clone());
        buttons
    };

    failsafe(failsafe_glyphs, failsafe_code_buttons);

    // Continuous async loop for program functions to keep repeating
    async fn main_loop(glyphs: Arc<Mutex<Vec<Button>>>, code_buttons: Arc<Mutex<Vec<Button>>>, execute_button: Button, codetext: Entry) {
        loop {
            // Creating clones of buttons to edit and adding to reset_glyphs
            let reset_glyphs = glyphs.lock().unwrap().clone();
            let reset_code_buttons = {
                let mut buttons = code_buttons.lock().unwrap().clone();
                buttons.push(execute_button.clone());
                buttons
            };
    
            // Creating clones of necessary glyphs and buttons to edit and adding to appropriate phase variables
            let phase_one_glyphs = glyphs.lock().unwrap().clone();
            let phase_two_glyphs = glyphs.lock().unwrap().clone();
            let phase_two_buttons = {
                let mut buttons = code_buttons.lock().unwrap().clone();
                buttons.push(execute_button.clone());
                buttons
            };
            let phase_three_glyphs = glyphs.lock().unwrap().clone();
            let phase_four_glyphs = glyphs.lock().unwrap().clone(); 

            // Creating clones of buttons to edit and adding to system_failure variables
            let system_failure_glyphs = glyphs.lock().unwrap().clone();
            let system_failure_buttons = {
                let mut buttons = code_buttons.lock().unwrap().clone();
                buttons.push(execute_button.clone());
                buttons
            };
            let system_failure_codetext = codetext.clone();
            
            // Main section of program which executes after variables required have been cloned
            let main_context_task = MainContext::default().spawn_local(async move {
                // Resetting SHOULD_RESET to base value (false)
                *SHOULD_RESET.lock().unwrap() = false;

                // Allows for reset and failsafe task to finish before executing next function
                if !*FAILSAFE.lock().unwrap() {
                    main_loop_with_timer(16).await;
                    *FAILSAFE.lock().unwrap() = true;
                } else {
                    main_loop_with_timer(1).await;
                }
                reset(reset_glyphs, reset_code_buttons).await;
        
                // Allows for wait task to finish before executing next function
                main_loop_with_timer(wait()).await;
        
                // Allows for phase one countdown to finish before executing next function
                phase_one(phase_one_glyphs).await;
                main_loop_with_timer(43).await;
                
                // Allows for phase two countdown to finish before executing next function
                phase_two(phase_two_glyphs, phase_two_buttons).await;   
                main_loop_with_timer(37).await;                     
        
                // Allows for phase three countdown to finish before executing next function, and checks to see if reset conditions are met
                if !*SHOULD_RESET.lock().unwrap() {
                    phase_three(phase_three_glyphs).await;
                    main_loop_with_timer(9).await;
                }
        
                // Allows for phase four countdown to finish before executing next function, and checks to see if reset conditions are met
                if !*SHOULD_RESET.lock().unwrap() {
                    phase_four(phase_four_glyphs).await;
                    main_loop_with_timer(12).await; 
                }
        
                // Allows for system failure function to finish before resetting program, and checks to see if reset conditions are met
                if !*SHOULD_RESET.lock().unwrap() {
                    system_failure(system_failure_glyphs, system_failure_buttons, system_failure_codetext).await;
                    main_loop_with_timer(15).await;
                }
                
                async_std::task::sleep(Duration::from_secs(1)).await;
            });

            // Handles execution and handling of main loop
            let result = main_context_task.await;
            match result {
                Ok(_) => {
                    eprintln!("Loop is running successfully...")
                },
                Err(err) => {
                    eprintln!("Error: {}", err);
                }
            }
        }
    }
    // Executes main loop to start taking place
    MainContext::default().spawn_local(main_loop(glyphs, code_buttons, execute_button, codetext_shared.lock().unwrap().clone()));
}