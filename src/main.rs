use std::{ffi::{CString, c_void}, fs, path::Path, process::Command, thread, time::{Duration, SystemTime, UNIX_EPOCH}};

use clap::Parser;
use futures::{StreamExt, stream::FuturesUnordered};
use indexmap::IndexMap;
use rand::{rngs::ThreadRng, seq::SliceRandom};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use wait::Waitable;
use windows::Win32::{UI::WindowsAndMessaging::{SPI_SETDESKWALLPAPER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoA}};

const CONFIG_DIR: &str = "C:\\39Choko\\SekaiBackground";
const CONFIG_PATH: &str = "C:\\39Choko\\SekaiBackground\\config.json";

#[derive(Deserialize, Serialize)]
struct Config {
    #[serde(flatten)]
    units: IndexMap<String, IndexMap<String, bool>>,
}

#[derive(clap::Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long, help = "Print config path", )]
    config: bool,

    #[arg(long, help = "Update SekaiBackground to the lastest version if an update is available")]
    update: bool,
}

impl Args {
    fn config(&self) {
        if self.config {
            println!("{}", CONFIG_PATH);
            std::process::exit(0);
        }
    }

    fn update(&self) {
        if self.update {
            println!("Checking for updates...");

            let current_verison = env!("CARGO_PKG_VERSION");
            let url = "https://api.github.com/repos/39Choko/SekaiBackground/releases/latest";
            let client = Client::new();
            let request = client.get(url).header("User-Agent", "SekaiBackground-Updater").send().wait();
            
            match request.into() {
                Ok(response) => {
                    if let Ok(text) = response.text().wait() {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                            let latest_version = json["tag_name"].as_str().unwrap_or("");

                            println!("Latest version: {}", latest_version);

                            if latest_version > current_verison {
                                println!("New version available: {}. Attemping to update...", latest_version);
                            } else {
                                println!("You are using the latest version.");
                                std::process::exit(0);
                            }
                        }
                    }

                }
                Err(_) => {
                    eprintln!("Failed to check for updates.");
                    std::process::exit(1);
                }
            }

            let status = Command::new("powershell.exe")
                .arg("-NoProfile")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-Command")
                .arg("Invoke-RestMethod -Uri https://raw.githubusercontent.com/39Choko/SekaiBackground/master/install.ps1 | Invoke-Expression")
                .spawn();

            match status {
                Ok(_) => {
                    println!("Updater started. Closing CLI to allow installation.");
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Failed to launch PowerShell: {}", e);
                    std::process::exit(1);
                },
            }
        }
    }
}

fn wait_for_network() {
    let client = Client::new();

    loop {
        if client.get("https://www.google.com").send().wait().is_ok() {
            break;
        }

        thread::sleep(Duration::from_secs(1));
    }
}

fn get_res_id(name: &str) -> Option<String> {
    let id = match name.to_lowercase().as_str() {
        "ichika" => 1,
        "saki" => 2,
        "honami" => 3,
        "shiho" => 4,
        "minori" => 5,
        "haruka" => 6,
        "airi" => 7,
        "shizuku" => 8,
        "kohane" => 9,
        "an" => 10,
        "akito" => 11,
        "toya" => 12,
        "tsukasa" => 13,
        "emu" => 14,
        "nene" => 15,
        "rui" => 16,
        "kanade" => 17,
        "mafuyu" => 18,
        "ena" => 19,
        "mizuki" => 20,
        "miku" => 21,
        "rin" => 22,
        "len" => 23,
        "luka" => 24,
        "meiko" => 25,
        "kaito" => 26,
        _ => return None,
    };
    Some(format!("res{:03}", id))
}

fn ensure_config_exists() {
    if !Path::new(CONFIG_DIR).exists() {
        fs::create_dir_all(CONFIG_DIR).expect("Failed to create config directory.");
    }

    if !Path::new(CONFIG_PATH).exists() {
        let mut units: IndexMap<String, IndexMap<String, bool>> = IndexMap::new();
        let groups: Vec<(&str, Vec<&str>)> = vec![
            (
                "Virtual Singer",
                vec!["miku", "rin", "len", "luka", "meiko", "kaito"],
            ),
            (
                "Leo/need", 
                vec!["ichika", "saki", "honami", "shiho"]
            ),
            (
                "MORE MORE JUMP!",
                vec!["minori", "haruka", "airi", "shizuku"],
            ),
            (
                "Vivid BAD SQUAD", 
                vec!["kohane", "an", "akito", "toya"]
            ),
            (
                "Wonderlands x Showtime",
                vec!["tsukasa", "emu", "nene", "rui"],
            ),
            (
                "Nightcord at 25:00",
                vec!["kanade", "mafuyu", "ena", "mizuki"],
            ),
        ];

        for (group_name, members) in groups {
            let mut char_map: IndexMap<String, bool> = IndexMap::new();
            for member in members {
                char_map.insert(member.to_string(), true);
            }
            units.insert(group_name.to_string(), char_map);
        }

        let config: Config = Config { units };
        let json: String = serde_json::to_string_pretty(&config).unwrap();
        fs::write(CONFIG_PATH, json).expect("Failed to write config.");
    }
}

fn set_wallpaper(path: &str) {
    unsafe {
        let path_c = CString::new(path).unwrap();
        SystemParametersInfoA(
            SPI_SETDESKWALLPAPER,
            0,
            Some(path_c.as_ptr() as *mut c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(3),
        )
        .unwrap_or_else(|_| eprintln!("Failed to set wallpaper."));
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    args.config();
    args.update();

    wait_for_network();
    ensure_config_exists();

    let config_data: String = fs::read_to_string(CONFIG_PATH).expect("Could not read config file.");
    let config: Config = serde_json::from_str(&config_data).expect("JSON Error");

    let mut pool: Vec<String> = Vec::new();
    for (_unit, members) in config.units {
        for (name, enabled) in members {
            if enabled {
                if let Some(res_id) = get_res_id(&name) {
                    pool.push(res_id);
                }
            }
        }
    }

    if pool.is_empty() {
        println!("No characters enabled in config.");
        return;
    }

    let client = Client::new();
    let mut tasks = FuturesUnordered::new();

    for _ in 0..15 {
        let client: Client = client.clone();

        let mut rng: ThreadRng = rand::thread_rng();
        let char_res: &String = pool.choose(&mut rng).unwrap();
        let card_number: u32 = (rand::random::<u32>() % 150) + 1;
        let url: String = format!(
            "https://storage.sekai.best/sekai-jp-assets/character/member/{}_no{:03}/card_after_training.webp",
            char_res, card_number
        );

        tasks.push(tokio::spawn(async move {
            let response: reqwest::Response = client.get(&url).send().await.map_err(|_| ())?;

            if response.status().is_success() {
                let bytes = response.bytes().await.map_err(|_| ())?;
                let img: image::DynamicImage = image::load_from_memory(&bytes).map_err(|_| ())?;
                Ok((img, url))
            } else {
                Err(())
            }
        }));
    }

    while let Some(result) = tasks.next().await {
        if let Ok(Ok((img, url))) = result {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            let bmp_path = std::env::temp_dir().join(format!("sekai_bg_{}.bmp", timestamp));
            img.save_with_format(&bmp_path, image::ImageFormat::Bmp)
                .expect("Failed to save BMP");

            let path_str = bmp_path.to_str().unwrap();
            set_wallpaper(path_str);
            println!("Wallpaper set from URL: {}", url);

            break;
        }
    }
}
