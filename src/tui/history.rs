use std::{
    collections::BTreeMap,
    fs::{self, create_dir_all, File},
    path::PathBuf,
};

use eyre::{eyre, OptionExt};
use ron::{
    de::from_reader,
    ser::{to_string_pretty, PrettyConfig},
};
use serde::{Deserialize, Serialize};

const HISTORY_FILE_NAME: &str = "history.ron";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
enum DownloadStage {
    READY,
    DOWNLOADING,
    MERGING,
    COMPLETE,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct HistoryDownload {
    file_name: String,
    url: String,
    stage_download: DownloadStage,
}

#[derive(Debug, Serialize, Deserialize)]
struct Histories {
    history: BTreeMap<u32, HistoryDownload>,
}

impl Histories {
    fn new(history_filename: &str) -> Self {
        let dir_path = Self::check_config_folder()
            .map_err(|err| println!("ERROR : {err}"))
            .expect("ERROR: error creating config folder");

        let file_path = Self::check_config_file(dir_path, history_filename);

        let history = match file_path {
            Some(file_path) => {
                let file_path = File::open(file_path).expect("ERROR : Opening config file");
                let hist: Histories = from_reader(file_path)
                    .map_err(|err| println!("ERROR: {err}"))
                    .unwrap();
                hist
            }
            None => {
                let map_history = BTreeMap::new();
                Self {
                    history: map_history,
                }
            }
        };
        history
    }

    fn swap_position(&mut self, num_a: u32, num_b: u32) -> eyre::Result<()> {
        let histo_a = self.get_history(num_a)?;

        let histo_b = self
            .history
            .insert(num_b, histo_a.clone())
            .ok_or_eyre("ERROR SWAP: history not exist")?;

        self.history.insert(num_a, histo_b);
        Ok(())
    }

    fn get_history(&self, num: u32) -> eyre::Result<&HistoryDownload> {
        let res = self
            .history
            .get(&num)
            .ok_or_else(|| eyre!("ERROR: No history with key number: {num}"))?;
        Ok(res)
    }

    fn update_stage(&mut self, num: u32, stage: DownloadStage) {
        let res = self
            .history
            .entry(num)
            .and_modify(|h| h.stage_download = stage);
    }

    fn add_history(&mut self, download_history: HistoryDownload) {
        let last = self
            .history
            .last_key_value()
            .and_then(|l| Some(*l.0))
            .unwrap_or_default();

        self.history.insert(last + 1, download_history);
    }

    fn save_history(&self, history_filename: &str) -> eyre::Result<()> {
        let file_path = Self::get_file_history(history_filename)?;
        let pretty_config = PrettyConfig::new().depth_limit(4).enumerate_arrays(true);
        let pretty_str = to_string_pretty(self, pretty_config)?;

        fs::write(&file_path, pretty_str)?;

        Ok(())
    }

    fn remove_history(&mut self, num: u32) -> Option<HistoryDownload> {
        self.history.remove(&num)
    }

    fn list(&self) -> &BTreeMap<u32, HistoryDownload> {
        &self.history
    }

    fn get_file_history(history_filename: &str) -> eyre::Result<PathBuf> {
        let dir_config = dirs::config_dir().ok_or_eyre("ERROR: config directory not available")?;

        let config_file = dir_config.join("tdm").join(history_filename);

        Ok(config_file)
    }

    fn check_config_folder() -> eyre::Result<PathBuf> {
        let dir_config = dirs::config_dir().ok_or_eyre("ERROR: config directory not available")?;

        let config_file = dir_config.join("tdm");

        if !config_file.exists() {
            create_dir_all(&config_file)?;
        }

        Ok(config_file)
    }

    fn check_config_file(path: PathBuf, history_filename: &str) -> Option<PathBuf> {
        let file_path = path.join(history_filename);

        let file_path = file_path.exists().then(|| file_path);

        file_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delete_file(history_filename: &str) -> eyre::Result<()> {
        let dir_config = dirs::config_dir().ok_or_eyre("ERROR: config directory not available")?;

        let config_file = dir_config.join("tdm").join(history_filename);
        fs::remove_file(config_file)?;

        Ok(())
    }

    #[test]
    fn test_history_insert() -> eyre::Result<()> {
        let history_filename = "history_test1.ron";
        let mut histories_arr = Histories::new(history_filename);

        assert_eq!(0, histories_arr.history.len());

        let download_history = HistoryDownload {
            file_name: "test.txt".to_owned(),
            url: "https://downlaod.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        histories_arr.add_history(download_history);

        assert_eq!(1, histories_arr.history.len());

        let histories_one = histories_arr.get_history(1)?;

        assert_eq!("test.txt", histories_one.file_name);

        Ok(())
    }

    #[test]
    fn test_history_save() -> eyre::Result<()> {
        let history_filename = "history_test2.ron";
        let mut histories_arr = Histories::new(history_filename);

        assert_eq!(0, histories_arr.history.len());

        let download_history = HistoryDownload {
            file_name: "test.txt".to_owned(),
            url: "https://downlaod.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        histories_arr.add_history(download_history);

        assert_eq!(1, histories_arr.history.len());

        histories_arr.save_history(history_filename)?;

        let histories_arr_2 = Histories::new(history_filename);

        let histo_1 = histories_arr.get_history(1)?;
        let histo_2 = histories_arr_2.get_history(1)?;

        let url_1 = &histo_1.url;
        let url_2 = &histo_2.url;

        delete_file(history_filename)?;
        assert_eq!(url_2, url_1);
        Ok(())
    }

    #[test]
    fn test_history_overwrite() -> eyre::Result<()> {
        let history_filename = "history_test3.ron";
        let mut histories_arr = Histories::new(history_filename);

        let download_history = HistoryDownload {
            file_name: "test.txt".to_owned(),
            url: "https://downlaod.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        histories_arr.add_history(download_history);
        histories_arr.save_history(history_filename)?;

        let download_history_1 = HistoryDownload {
            file_name: "test1.txt".to_owned(),
            url: "https://downlaod1.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        histories_arr.add_history(download_history_1);
        histories_arr.save_history(history_filename)?;

        let history_test = Histories::new(history_filename);

        delete_file(history_filename)?;
        assert_eq!(2, history_test.history.len());

        Ok(())
    }

    #[test]
    fn test_history_swap() -> eyre::Result<()> {
        let mut histo_arr = Histories::new("dump.ron");
        let download_history = HistoryDownload {
            file_name: "test.txt".to_owned(),
            url: "https://downlaod.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        let download_history_1 = HistoryDownload {
            file_name: "test1.txt".to_owned(),
            url: "https://downlaod1.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        let download_history_2 = HistoryDownload {
            file_name: "test2.txt".to_owned(),
            url: "https://downlaod2.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        let download_history_3 = HistoryDownload {
            file_name: "test3.txt".to_owned(),
            url: "https://downlaod3.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        histo_arr.add_history(download_history);
        histo_arr.add_history(download_history_1);
        histo_arr.add_history(download_history_2);
        histo_arr.add_history(download_history_3);

        histo_arr.swap_position(2, 3)?;

        let histo_2 = histo_arr.get_history(2)?;
        assert_eq!("https://downlaod2.com", histo_2.url);

        let histo_3 = histo_arr.get_history(3)?;
        assert_eq!("https://downlaod1.com", histo_3.url);

        Ok(())
    }

    #[test]
    fn test_history_remove() {
        let mut histo_arr = Histories::new("dump.ron");
        let download_history = HistoryDownload {
            file_name: "test.txt".to_owned(),
            url: "https://downlaod.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        let download_history_1 = HistoryDownload {
            file_name: "test1.txt".to_owned(),
            url: "https://downlaod1.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        histo_arr.add_history(download_history);
        histo_arr.add_history(download_history_1);

        histo_arr.remove_history(2);

        assert_eq!(1, histo_arr.list().len());

        let res = histo_arr.get_history(2);

        assert!(res.is_err());
    }

    #[test]
    fn test_history_update() -> eyre::Result<()> {
        let mut histo_arr = Histories::new("dump.ron");
        let download_history = HistoryDownload {
            file_name: "test.txt".to_owned(),
            url: "https://downlaod.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        histo_arr.add_history(download_history);

        let merging = DownloadStage::MERGING;

        histo_arr.update_stage(1, merging.clone());

        let stage = &histo_arr.get_history(1)?.stage_download;

        assert_eq!(merging, stage.clone());
        Ok(())
    }

    #[test]
    fn test_history_test() {
        let mut histo_arr = Histories::new("dump.ron");
        let download_history = HistoryDownload {
            file_name: "test.txt".to_owned(),
            url: "https://downlaod.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        let download_history_1 = HistoryDownload {
            file_name: "test1.txt".to_owned(),
            url: "https://downlaod1.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        let download_history_2 = HistoryDownload {
            file_name: "test2.txt".to_owned(),
            url: "https://downlaod2.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        let download_history_3 = HistoryDownload {
            file_name: "test3.txt".to_owned(),
            url: "https://downlaod3.com".to_owned(),
            stage_download: DownloadStage::READY,
        };

        histo_arr.add_history(download_history);
        histo_arr.add_history(download_history_1);
        histo_arr.add_history(download_history_2);
        histo_arr.add_history(download_history_3);

        let res = histo_arr.list().iter().collect::<Vec<_>>();

        assert!(res.len() > 0);
    }
}
