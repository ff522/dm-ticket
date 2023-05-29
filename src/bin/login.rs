use anyhow::Result;
use dm_ticket::client::LoginClient;
use dotenv::dotenv;

use log::{debug, error, info};
use std::{
    env,
    io::{self, Write},
    time::Duration,
};

pub struct DmLogin {
    client: LoginClient,
}

impl DmLogin {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            client: LoginClient::new().await?,
        })
    }

    pub async fn run(&self) -> Result<()> {
        info!("正在获取二维码...\n");
        let qrcode_data = match self.client.generate_qrcode().await {
            Ok(data) => {
                debug!("Get qrcode data:{:?}", data);
                data
            }
            Err(e) => {
                error!("Fail to get qrcode data, error:{:?}", e);
                return Err(e);
            }
        };

        let qrcode = match self.client.get_qrcode(qrcode_data.code_content).await {
            Ok(code) => {
                debug!("success to get qrcode!");
                code
            }
            Err(e) => {
                error!("Fail to get qrcode, error:{:?}", e);
                return Err(e);
            }
        };

        println!("{}\n", qrcode.to_str());

        let t = qrcode_data.t;
        let ck = qrcode_data.ck.clone();

        let max_times = 60 * 5;

        for i in 0..max_times {
            let qrcode_scan_status = self.client.get_login_result(t, ck.clone()).await?;

            match qrcode_scan_status.qrcode_status.as_str() {
                "NEW" => {
                    print!("\r请使用大麦APP扫码, 倒计时:{}秒\t", max_times - i);
                    let _ = io::stdout().flush();
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                "SCANED" => {
                    print!("\r##########请点击确认登录#######\t\t");
                    let _ = io::stdout().flush();
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                "CONFIRMED" => {
                    println!("\r\n已确认, 登录成功, cookie如下: ");
                    let cookie2 = qrcode_scan_status.cookie2.unwrap();
                    let return_url = qrcode_scan_status.return_url.unwrap();
                    let st = qrcode_scan_status.st.unwrap();
                    let cookies = self.client.get_cookie(cookie2, return_url, st).await?;
                    println!("\n{}\n", cookies);
                    return Ok(());
                }
                "EXPIRED" => {
                    print!("\r\t二维码已过期, 请重新执行程序...");
                    return Ok(());
                }
                _ => {
                    error!("未知状态:{:?}, 退出...", qrcode_scan_status);
                    return Ok(());
                }
            }
        }

        info!("二维码已过期, 请重新执行程序...");

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO");
    }

    if env::var("TOKEN_SERVER_URL").is_err() {
        env::set_var("TOKEN_SERVER_URL", "http://127.0.0.1:8080/");
    }

    if env::var("QRCODE_PATH").is_err() {
        env::set_var("QRCODE_PATH", "./qrcode.png");
    }

    pretty_env_logger::init();

    let app = DmLogin::new().await?;
    app.run().await?;

    Ok(())
}
