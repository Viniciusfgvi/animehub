use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use animehub::integrations::mpv::client::MpvClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let video = std::env::args()
        .nth(1)
        .expect("Passe o caminho do vídeo como argumento");

    let video_path = PathBuf::from(video);

    // Cria o cliente MPV
    let client = MpvClient::new()?;

    // Lança o MPV e carrega o vídeo
    client.launch(video_path)?;
    println!("MPV launched. Pausando em 2s...");

    sleep(Duration::from_secs(2));

    // Usa API pública (NUNCA send_command)
    client.pause()?;
    println!("Pausado. Retomando em 2s...");

    sleep(Duration::from_secs(2));

    client.resume()?;
    println!("Resumed. Parando em 3s...");

    sleep(Duration::from_secs(3));

    client.stop()?;
    println!("MPV stopped.");

    Ok(())
}
