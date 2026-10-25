use super::crypto::*;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use serde_json::Value;
use std::time::Duration;

pub fn endpoint(url: &str, path: &str) -> String {
    format!("{}/{}", url.trim_end_matches('/'), path.trim_start_matches('/'))
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("构造 HTTP 客户端失败")
}

pub async fn post_encrypted(
    url: &str,
    plaintext: &[u8],
    uuid: &str,
    device_name: &str,
    token: &str,
) -> Option<Value> {
    let client = http_client();
    let sm4_key_b64 = gen_sm4_key_b64();
    let sm4_key = B64.decode(&sm4_key_b64).ok()?;
    let cipher_key = sm2_encrypt(&sm4_key_b64);
    let content = sm4_encrypt(plaintext, &sm4_key);
    let body = serde_json::json!({ "cipherKey": cipher_key, "content": content });
    let host = url_host(url);
    let headers = build_headers(uuid, device_name, token, &host);

    for _ in 0..3 {
        match client.post(url).headers(headers.clone()).json(&body).send().await {
            Ok(resp) => {
                let status = resp.status();
                let clip = |s: &str| {
                    let n = s.chars().count();
                    let c: String = s.chars().take(1200).collect();
                    if n > 1200 { format!("{}…(共{}字符)", c, n) } else { c }
                };
                if let Ok(text) = resp.text().await {
                    let trimmed = text.trim();

                    if !status.is_success() {
                        eprintln!("[http] {} 非成功状态 {} raw={}", url, status, clip(trimmed));
                        if status.as_u16() >= 500 {
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
                            return Some(v);
                        }
                        return Some(Value::String(trimmed.to_string()));
                    }
                    let v = sm4_decrypt(&sm4_key, trimmed);
                    if v.is_null() {
                        eprintln!("[http] {} 解密/解析失败 raw(len={})={}", url, trimmed.len(), clip(trimmed));
                    } else {
                        eprintln!("[http] {} => code={:?} msg={:?} body={}", url, v.get("code"), v.get("msg"), clip(&v.to_string()));
                    }
                    return Some(v);
                }
                return None;
            }
            Err(e) => {
                eprintln!("[http] {} 请求异常: {}", url, e);
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        }
    }
    None
}

pub async fn list_schools(uuid: &str, device_name: &str) -> Vec<Value> {
    let url = SCHOOL_LIST_URL;
    let host = url_host(url);
    let headers = build_headers(uuid, device_name, "", &host);
    let client = http_client();
    if let Ok(resp) = client.post(url).headers(headers).send().await {
        if let Ok(v) = resp.json::<Value>().await {
            if v.get("code").and_then(|c| c.as_i64()) == Some(200) {
                if let Some(arr) = v.get("data").and_then(|d| d.as_array()) {
                    return arr.clone();
                }
            }
        }
    }
    vec![]
}

pub async fn post_json(
    url: &str,
    value: &Value,
    uuid: &str,
    device_name: &str,
    token: &str,
) -> Option<Value> {
    let bytes = serde_json::to_vec(value).ok()?;
    post_encrypted(url, &bytes, uuid, device_name, token).await
}

pub async fn post_bytes(
    url: &str,
    bytes: &[u8],
    uuid: &str,
    device_name: &str,
    token: &str,
) -> Option<Value> {
    post_encrypted(url, bytes, uuid, device_name, token).await
}

pub async fn post_empty(url: &str, uuid: &str, device_name: &str, token: &str) -> Option<Value> {
    post_encrypted(url, b"", uuid, device_name, token).await
}
