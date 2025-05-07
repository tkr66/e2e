use serde::Deserialize;
use thirtyfour::{ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};

use super::Window;

#[derive(Debug, Deserialize)]
pub struct Driver {
    pub host: String,
    pub port: String,
    pub headless: bool,
    pub window: Window,
}

impl Driver {
    pub async fn initialize(&self) -> Result<WebDriver, Box<dyn std::error::Error>> {
        let mut caps = DesiredCapabilities::edge();
        if self.headless {
            caps.set_headless()?;
        }
        let driver_url = format!("http://{}:{}", self.host, self.port);
        let driver = WebDriver::new(driver_url, caps).await?;
        let window = &self.window;
        driver
            .set_window_rect(window.x, window.y, window.width, window.height)
            .await?;
        Ok(driver)
    }
}
