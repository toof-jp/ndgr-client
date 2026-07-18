# ndgr-client

niconico Live comment viewer

## Structure

- `cli/` — Rust 製 TUI コメントビューワー (`ndgr-client`)
- `protobuf/` — NDGR protobuf 定義の Rust バインディング (git submodule を含む)
- `web/` — ブラウザで動く静的サイト版コメントビューワー (React + [Vite+](https://viteplus.dev/) + wasm)
  - `web/wasm/` — `ndgr-client` を wasm-bindgen でラップした wasm クレート

## Dependency

To build, you need to install `protoc` (Protocol Buffer Compiler).
https://docs.rs/prost-build/latest/prost_build/#sourcing-protoc

## CLI

```sh
cargo run -p ndgr-client -- https://live.nicovideo.jp/watch/lvXXXXXXXX
```

## Web

Requires [wasm-pack](https://rustwasm.github.io/wasm-pack/) and the
`wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`).

```sh
cd web
vp install
vp run wasm    # wasm-pack build wasm --target web --out-dir pkg
vp dev         # development server
vp run build   # static site → web/dist/
```

`web/dist/` は任意の静的ホスティングにそのまま配置できます。

> [!NOTE]
> 番組ページ (`live.nicovideo.jp`) の HTML 取得はブラウザの CORS 制限で
> ブロックされる場合があります。その場合は UI の「CORSプロキシ」欄に
> リクエスト URL の前に連結するプロキシの URL prefix を指定してください
> (NDGR メッセージサーバーへのリクエストにも同じ prefix が適用されます)。

## Related Projects

- https://github.com/n-air-app/nicolive-comment-protobuf
- https://github.com/rinsuki-lab/ndgr-reader
- https://github.com/tsukumijima/NDGRClient
