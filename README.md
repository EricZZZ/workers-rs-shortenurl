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
安装 `pre-commit`
