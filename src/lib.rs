use std::str::FromStr;

use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use worker::*;

const DEFAULT_LIMIT: i32 = 10;

// 定义常用httpcode状态码

const HTTP_BAD_REQUEST: u16 = 400;
const HTTP_NOT_FOUND: u16 = 404;
const HTTP_INTERNAL_SERVER_ERROR: u16 = 500;
const HTTP_SERVICE_UNAVAILABLE: u16 = 503;

#[derive(Serialize, Deserialize, Debug)]
struct Shortenurl {
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ShortenReq {
    url: String,
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .post_async("/shorten", |mut req, ctx| async move {
            let url = match req.json::<Shortenurl>().await {
                Ok(c) => c.url,
                Err(_) => String::from(""),
            };
            if url.is_empty() {
                return Response::error("Bad Request", HTTP_BAD_REQUEST);
            };
            // 获得当前时间 转换成yyyy-mm-dd
            let now = chrono::Local::now();
            let time = now.format("%Y-%m-%d").to_string();
            console_log!("time: {}", time);

            // 获取环境每日limit变量

            let limit: i32 = match ctx.env.var("LIMIT") {
                Ok(c) => c.to_string().parse().unwrap(),
                Err(_) => DEFAULT_LIMIT,
            };

            let count = match ctx.kv("shortenurl")?.get(&time).text().await? {
                Some(count) => {
                    let count: i32 = count.parse().unwrap();

                    count
                }
                None => 0,
            };
            console_log!("count: {}/n limit: {}", count, limit);
            if count >= limit {
                return Response::error("Exceeding the limit", HTTP_SERVICE_UNAVAILABLE);
            }
            let id = nanoid!(6);
            match ctx.kv("shortenurl")?.put(&id, &url)?.execute().await {
                Ok(_) => {
                    match ctx
                        .kv("shortenurl")?
                        .put(&time, (count + 1).to_string())?
                        .execute()
                        .await
                    {
                        Ok(_) => {}
                        Err(_) => {
                            return Response::error(
                                "server internal error",
                                HTTP_INTERNAL_SERVER_ERROR,
                            );
                        }
                    }
                    let short_url = Shortenurl {
                        url: format!(
                            "https://{}{}{}",
                            req.url()?.host_str().unwrap(),
                            "/shorten/",
                            id
                        ),
                    };
                    Response::from_json(&short_url)
                }
                Err(_) => Response::error("Bad Request", HTTP_BAD_REQUEST),
            }
        })
        .get_async("/shorten/:id", |_req, ctx| async move {
            if let Some(id) = ctx.param("id") {
                return match ctx.kv("shortenurl")?.get(id).text().await? {
                    Some(a) => {
                        let url = Url::from_str(a.as_str()).unwrap();
                        Response::redirect(url)
                    }
                    None => Response::error("url not found", HTTP_NOT_FOUND),
                };
            }
            Response::error("Bad Request", HTTP_BAD_REQUEST)
        })
        .run(req, env)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_nanoid() {
        let id = nanoid!(6);
        assert_eq!(id.len(), 6);
    }
}
