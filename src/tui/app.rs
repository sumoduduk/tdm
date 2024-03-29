mod history;

use crate::{utils::to_vec::string_to_vec, DownloadStage, HistoryDownload};
use tui_input::Input;

use self::history::Histories;

const HISTORY_FILE_NAME: &str = "history.ron";

pub enum CurrentScreen {
    Main,
    Editing,
    Setting,
    Exiting,
}

pub enum InputMode {
    Normal,
    Editing,
}

pub struct AppTui {
    pub input_uri: Input,
    pub input_mode: InputMode,
    pub curr_screen: CurrentScreen,
    pub saved_input: Vec<String>,
    pub history: Histories,
}

impl AppTui {
    pub fn new() -> Self {
        Self {
            input_uri: Input::default(),
            input_mode: InputMode::Normal,
            curr_screen: CurrentScreen::Main,
            saved_input: Vec::new(),
            history: Histories::new(HISTORY_FILE_NAME),
        }
    }

    pub fn save_input(&mut self) {
        let input_value = self.input_uri.value();

        if input_value.contains(',') {
            let mut vec_str = string_to_vec(input_value);
            self.saved_input.append(&mut vec_str);
        } else {
            self.saved_input.push(input_value.into());
        }

        self.input_uri.reset();
    }

    pub fn print_vec(&self) -> eyre::Result<()> {
        let output = serde_json::to_string_pretty(&self.saved_input)?;
        println!("{}", output);
        Ok(())
    }

    pub fn add_history(&mut self, download_history: HistoryDownload) -> u32 {
        let key = self.history.add_history(download_history);
        key
    }

    pub fn update_stage(&mut self, num: u32, stage: DownloadStage) {
        self.history.update_stage(num, stage);
    }
    pub fn save_history(&self) -> eyre::Result<()> {
        self.history.save_history(HISTORY_FILE_NAME)?;
        Ok(())
    }
}
