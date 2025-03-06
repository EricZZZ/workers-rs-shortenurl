# 使用 Cloudflare Workers-rs 开发短链接 Api 服务

## 准备工作
使用模版创建项目

```bash
cargo generate cloudflare/workers-rs
```

选择 `axum` 模版，并安装 wrangler

```bash
npm install wrangler --save-dev
```

本地部署测试

```bash
npx wrangler dev
```
请求 `http:://127.0.0.1:8787/` 可以看到返回结果

部署到 cloudflare 上

```bash
# configure your routes, zones & more in your worker's `wrangler.toml` file
npx wrangler deploy
```
安装 `pre-commit`，提交前检查代码格式
```bash
pre-commit install
```

配置 cloundflare kv 存储
```bash
npx wrangler kv namespace create shortenurl
```
在 wrangler.toml 文件中，绑定 kv 存储
```toml
[[kv_namespaces]]
binding = "<BINDING_NAME>"
id = "<BINDING_ID>"
```

## 功能描述
### 短链生成
通过 `POST` 请求，传入长链，返回短链
```bash
curl --request POST \
  --url http://localhost:8787/shorten \
  --header 'content-type: application/json' \
  --header 'user-agent: vscode-restclient' \
  --data '{"url":"https://medium.com/@r.das699/an-example-of-connecting-to-a-sqlite-database-using-rust-cdeb363a277a"}'
```
返回结果
```json
{
    url:"http://localhost:8787/shorten/2EHg4q"
}
```

### 短链跳转
通过 `GET` 请求，传入短链，返回长链
```bash
curl --request GET \
  --url http://localhost:8787/shorten/2EHg4q \
  --header 'user-agent: vscode-restclient'
```
返回结果，跳转至长链

## 开发
使用 `nanoid` creat，生成短链字符串， 需要配置 `getrandom` crate js 支持
``` toml
[dependencies]
getrandom = { version = "0.2", features = ["js"] }
```

开启 npm 代理（或许会用到）
```bash

// 查看代理
npm config get proxy
npm config get https-proxy

// 设置代理
npm config set proxy http://127.0.0.1:7890
npm config set https-proxy http://127.0.0.1:7890

// 删除代理
npm config delete proxy
npm config delete https-proxy

```

本来要使用 `axum` 开发，但是没搞明白，异步下 env 如何在多个线程共享（卡了好几天😅），先用 cloudflare 原生`http`来实现了。

1. 尝试 Using the State extractor 没成功 ❌
2. 尝试 Using request extensions 没成功 ❌ （ps. 报错看不懂😭）
3. 尝试使用别人写好的 [crate](https://crates.io/crates/axum-cloudflare-adapter)，通过 crate 中的方法

```rust
#[wasm_compat]
async fn index(State(state): State<AxumState>) -> Html<&'static str> {
    let env: &Env = state.env_wrapper.env.deref();
    let worker_rs_version: Var = env.var("WORKERS_RS_VERSION").unwrap();
    console_log!("WORKERS_RS_VERSION: {}", worker_rs_version.to_string());
    Html("<p>Hello from Axum!</p>")
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    let mut _router: AxumRouter = AxumRouter::new()
        .route("/", get(index))
        .with_state(AxumState {
            env_wrapper: EnvWrapper::new(env),
        });
    let axum_request = to_axum_request(req).await.unwrap();
    let axum_response = _router.call(axum_request).await.unwrap();
    let response = to_worker_response(axum_response).await.unwrap();
    Ok(response)
}
```
可以获取到`env`中的环境变量，但是使用`kv`，无法使用其异步方法，最后没成功（水平不够😮‍💨）。

## 总结
初次尝试使用 `workers-rs` 写 Api 服务，目的熟悉`Rust`语言，而且想试试`cloudflare` 与 `axum`框架能擦出什么样的火花，结果跟我想的完全不一样，与直接使用`axum`相比，`workers-rs`的写法更加独特，它与 axum 的联系好像不像写`axum`那样自然，而起`workers-rs`编译过后是 wasm，对异步支持不是很好，整个`tokio`生态用不起来，不过通过这个小项目，对`Rust`语法更加熟悉了，编译器是最大对手，对不同框架是如何结合使用也有了一些研究，等以后弄懂了`http`，`axum`再来尝试使用 axum 的写法。
