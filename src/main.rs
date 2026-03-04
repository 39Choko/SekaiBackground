use std::{ffi::{CString, c_void}, fs, path::Path};

use futures::{StreamExt, stream::FuturesUnordered};
use indexmap::IndexMap;
use rand::{rngs::ThreadRng, seq::SliceRandom};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use windows::Win32::UI::WindowsAndMessaging::{SPI_SETDESKWALLPAPER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoA};

const CONFIG_DIR: &str = "C:\\39Choko\\SekaiBackground";
const CONFIG_PATH: &str = "C:\\39Choko\\SekaiBackground\\config.json";

#[derive(Deserialize, Serialize)]
struct Config {
    #[serde(flatten)]
    units: IndexMap<String, IndexMap<String, bool>>,
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
            let bmp_path = std::env::temp_dir().join("sekai_bg.bmp");
            img.save_with_format(&bmp_path, image::ImageFormat::Bmp)
                .expect("Failed to save BMP");

            let path_str = bmp_path.to_str().unwrap();
            set_wallpaper(path_str);
            println!("Wallpaper set from URL: {}", url);

            break;
        }
    }
}
