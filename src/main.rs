use chrono::*;
use voskrust::api::*;
use voskrust::sound::*;

const HOST: &'static str = "http://localhost:70/";
const MODEL: &'static str = "./vosk-model-small-ru-0.22";
const TIME_RANGE_START_HOUR: u32 = 1;
const TIME_RANGE_END_HOUR: u32 = 18;

const WORDED_ACTIONS: &[(&'static str, LedAction)] = &[
    ("свет", LedAction::Power(true)),
    ("тьма", LedAction::Power(false)),
    (
        "белый",
        LedAction::Color {
            red: 0,
            green: 0,
            blue: 0,
        },
    ),
    (
        "фиолетовый",
        LedAction::Color {
            red: 255,
            green: 0,
            blue: 255,
        },
    ),
    (
        "пурпурный",
        LedAction::Color {
            red: 128,
            green: 0,
            blue: 128,
        },
    ),
    (
        "красный",
        LedAction::Color {
            red: 255,
            green: 0,
            blue: 0,
        },
    ),
    (
        "зелёный",
        LedAction::Color {
            red: 0,
            green: 255,
            blue: 0,
        },
    ),
    (
        "голубой",
        LedAction::Color {
            red: 205,
            green: 92,
            blue: 92,
        },
    ),
    (
        "синий",
        LedAction::Color {
            red: 0,
            green: 0,
            blue: 255,
        },
    ),
];

enum LedAction {
    Color { red: u8, green: u8, blue: u8 },
    Power(bool),
}

impl LedAction {
    fn perform(&self) {
        let link = format!(
            "{host}{action}",
            host = HOST,
            action = match self {
                LedAction::Color { red, green, blue } =>
                    format!("color/{r}/{g}/{b}", r = red, g = green, b = blue,),
                LedAction::Power(state) => format!(
                    "power/{state}",
                    state = if *state { "true" } else { "false" },
                ),
            }
        );
        _ = reqwest::blocking::Client::new().put(link).send();
    }
}

fn main() {
    set_log_level(1);
    let model = Model::new(MODEL).unwrap();

    let mut recognizer = None;
    let mut audioreader = None;
    loop {
        let current_hour = Local::now().hour();
        if current_hour >= TIME_RANGE_START_HOUR && current_hour < TIME_RANGE_END_HOUR {
            recognizer = None;
            audioreader = None;
            std::thread::sleep(std::time::Duration::from_secs(60));
            continue;
        }
        if recognizer.is_none() {
            recognizer = Some(Recognizer::new(&model, 16000f32));
        }
        if audioreader.is_none() {
            audioreader = Some(ParecStream::init().unwrap());
        }
        let buf = {
            let audioreader = audioreader.as_mut().unwrap();
            audioreader.read_n_milliseconds(100.0).unwrap()
        };

        let text = {
            let recognizer = recognizer.as_mut().unwrap();
            if recognizer.accept_waveform(&buf[..]) {
                recognizer.final_result()
            } else {
                recognizer.partial_result()
            }
        };

        if text.is_empty() {
            continue;
        }

        for (action_word, action) in WORDED_ACTIONS {
            if !text.contains(action_word) {
                continue;
            }
            recognizer = None;
            std::thread::spawn(|| action.perform());
            break;
        }
    }
}
