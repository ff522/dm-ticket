use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
pub struct QrcodeContentGetParams {}

impl QrcodeContentGetParams {
    pub fn build() -> Result<serde_json::Value> {
        let mut rng = thread_rng();
        let csrf_token: String = (0..21).map(|_| rng.sample(Alphanumeric) as char).collect();
        let umid_token: String = (0..40).map(|_| rng.sample(Alphanumeric) as char).collect();
        let hsiz: String = (0..10).map(|_| rng.sample(Alphanumeric) as char).collect();
        let hsiz = format!("{:?}", md5::compute(hsiz)).to_ascii_lowercase();
        Ok(json!({
            "appName": "damai",
            "fromSite": "18",
            "appEntrance": "damai",
            "_csrf_token": csrf_token,
            "umidToken": umid_token.to_lowercase(),
            "isMobile": "false",
            "lang": "zh_CN",
            "returnUrl": "https://passport.damai.cn/dologin.htm?redirectUrl=https%253A%252F%252Fwww.damai.cn%252F&platform=106002",
            "hsiz": hsiz,
            "bizParams": "",
            "umidTag": "SERVER",
            "_bx-v": "2.5.0",
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QrcodeData {
    pub t: u64,

    #[serde(rename = "codeContent")]
    pub code_content: String,

    pub ck: String,

    #[serde(rename = "resultCode")]
    pub result_code: u32,
}

pub struct QrCodeLoginGetResParams {}

impl QrCodeLoginGetResParams {
    pub fn build() -> Result<Value> {
        Ok(json!({
            "appName": "damai",
            "fromSite": "18",
            "_bx-v":	"2.5.0",
        }))
    }
}

pub struct QrCodeLoginGetResForm {}

impl QrCodeLoginGetResForm {
    pub fn build(t: u64, ck: String) -> Result<Value> {
        let mut rng = thread_rng();
        let csrf_token: String = (0..21).map(|_| rng.sample(Alphanumeric) as char).collect();
        let umid_token: String = (0..40).map(|_| rng.sample(Alphanumeric) as char).collect();
        let hsiz: String = (0..10).map(|_| rng.sample(Alphanumeric) as char).collect();
        let hsiz = format!("{:?}", md5::compute(hsiz)).to_ascii_lowercase();
        let page_trace_id: String = (0..40).map(|_| rng.sample(Alphanumeric) as char).collect();
        let device_id: String = (0..24).map(|_| rng.sample(Alphanumeric) as char).collect();
        let ua = "140#5kTduXsozzW+Aewfweo23zaT4pN8s77o4RXRIbLoFyw9NRhkRYvtS/4OctmocZ9Cs0AEwJMDXuJqlbzxh31jNxOBzFcbs2c9lpTzzPzbVXlqlbnzrk82tQQLl6bydCi6u2sd1mMy/4hjXObdYFq23fAzl7MydD174LFdlKuvn6CXDpMnORKHtDXi1qXv4bMH4B2nQbfvr1VXwhQn+EcHcldUOuF0afRzPRuc8Ls0j1oRLR+c+ZcmWvp6UTQshAdtx/X+JG1sr/WW+KO+cY8xWSexILbspEOUrmz+QETswRlPCsS3dXjzcP1VT/1pnUySn4X3OHQpF3mPBGb3Viqz40AOlaMprMGxF2b4hOuJ3+CFogb477wDwWfYg2TJzZbCHB+4yZMJu3mFxYS4+Nko+Qsd96ONvCxk+3FKIBsNFulZFGrKyFIon0i0NjzNsP+pmKS2hksOnsmCDg+2+9YdKEbQSOzOVb+m3Hr21FOOu+lCqa+2JWIdqZbZvpz9xe3jrKfL7Ls94towPOFLrHNrwf3H1h+9iWfT/3SLVqz7W2WfAtfmbRccfIusEku71bMcXKMmNb27d2gfwmQmsWKcFntTO+faySt3F91He9+aognA/NFH2PPKV66ezzrb2Xe8r0wAONdOHaU3/0KBb7uu0oGGEUQ4zHWq9Rafxi2W+aD4KJeTMTAnykbFd/81MJFh0z38CZG72DC5Ml59C9zoi6cE3NcmpNxg+nOOP/bGCERdFbtsZLGADUDiHU690c5qzv6LgbqAV6f6Q2I+P+ArfqWu20V9dLAMz1nrHQ8efMQLRxvQBIxf3zB0fbc8IQ1gK/hXjFfF22u0NzJtpEk9gL9tbUhZ4T6ePh2GGcndSKoYQBbp2vAXiA78ju1qWYArzkhBWpEWQ+xMabDg3Wpbq6s3lhFYcdnrPUj122nb6pTZwqqm0Zh37dLAzxhHOpiVDcsTPlyqoMTOGvFMeNmqmUKVp1v8Vyqrronfoq/l+IBbU1Yg6I0hPQbDMyhdBdwrtPlySsWHU79k22k054q8JEY5iv5Ajv348XNE0osS5ATxeq1YgqkGlJq1371U2RMRPh9kYqTGloVab7fXo1tbC12/zI9A+3vYuZk6f34418v9/jklfYRCGsgeGi2ElfiDkGF3frEM1JghUU7VHU991TJCKNFUhPPX2MhOTjSiDRyaJ5HsFlb8zSK1deU1SPmR/HO57dzCZ+8txKpSRbJvpCdKmHM4duGN0gaWdKX8m7cxRtjitACkvrnyjzp4EOIXHO+wIkz5UuAKgLpQ7dtmSIHCHFNQ9LGS5xqcVzm1BcdZLeCzJjS4EZofyRTjbYq7BbJlqFAai93mxTtAbXpztIlxM8BYCe17Uz7rCZxRJwvVtzqEKuS9GRn9fCyDYsoSUtQUb1xevim+AQOPc5DO14+sQosqpLB65XELHQ/H4RXOTMwi23CUoivNXP2KkzVbqLunhbGJuzZDHB9RTKsMRNIfSnSQBzHQ8fnyky8LhjgkLAHG65X4cIbBKZVFR0jQVZLK80gpqvlmE+biVOPezteVmcqheeASnr0bjfJnN7jddaAXzmDirIU1+O0vb0wxmVFmqWAlpYOStgR2CZguHdGGrYCdsAQLDgiPulnLZPufSeR3GNldmnqZHli+FBloV4fEDIIHMlA9Ykgsc2UNZ0iBRiVxZoJDQHR1dnutQpO3XWh8aWwcSK0aepay6MZc8QshNbDDSU9fWXXG7t9+cpqNEmdtNH8dNw9MJP0QLR23G+M7MVQSc8LE0m9JT2CCrI6/6pU3gfIfx+GLzkcnNN2OuBaYjoysWAiea62wxlgHQ1xMQdEo8ZY/x/xTZ7HR";
        let form = json!({
             "t": t.to_string(),
             "ck": ck.as_str(),
             "ua": ua,
             "appName": "damai",
             "appEntrance": "damai",
             "fromSite": "18",
             "_csrf_token": csrf_token,
             "umidToken": umid_token.to_lowercase(),
             "isMobile": "false",
             "lang": "zh_CN",
             "returnUrl": "https://passport.damai.cn/dologin.htm?redirectUrl=https%253A%252F%252Fwww.damai.cn%252F&platform=106002",
             "hsiz": hsiz,
             "bizParams": "",
             "umidTag": "SERVER",
             "navlanguage": "zh-CN",
             "navUserAgent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36",
             "navPlatform": "MacIntel",
             "deviceId": device_id,
             "pageTraceId": page_trace_id.to_lowercase(),
        });
        Ok(form)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QrCodeLoginStatusData {
    #[serde(rename = "resultCode")]
    pub result_code: u32,

    #[serde(rename = "qrCodeStatus")]
    pub qrcode_status: String,

    pub st: Option<String>,

    #[serde(rename = "loginType")]
    pub login_type: Option<String>,

    #[serde(rename = "loginScene")]
    pub login_scene: Option<String>,

    pub sid: Option<String>,

    pub cookie2: Option<String>,

    #[serde(rename = "returnUrl")]
    pub return_url: Option<String>,
}
