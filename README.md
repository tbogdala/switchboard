# Switchboard! - v0.0.1

Aiming to be a lightweight, modern looking and easy to deploy chat interface to large language models
that can be used as-is and easily self-hosted, themed and modified as needed.

Note: Everything is under heavy development right now; Consider any data as disposable as log compatability isn't guaranteed yet.

[Try the Live Demo Site!](https://tbogdala.github.io/switchboard/)


## Features

* Clean and Simple: Setup your API settings and you're good to go!

* Themable: Clone this repo and change [tailwind-import.css](./tailwind-import.css) to have the webpage look
  how you want it to look. Or fork and update the modular components for deeper customization.

* Supports OpenAI-compatbile APIs such as [llama.cpp's server](https://github.com/ggml-org/llama.cpp), which when
  combined with [llama-swap](https://github.com/mostlygeek/llama-swap), will allow multiple models to be served.
  It also works with popular cloud API providers such as [openrouter](https://openrouter.ai/).

* Save slots allow you to save your conversation in the web browser's storage, meaning no file management
  is required. 

* Chatlogs can be imported and exported using JSON.


## Requirements

Building from source requires the `git` and `tailwindcss` executables to be in your path, in 
addition to having a [Rust toolchain](https://www.rust-lang.org/) installed.


## Building and Running

Once you have a Rust toolchain, make sure the `tailwindcss` executable is in your
path becausae it's referenced in the build scripts. The build process will also expect `git`
to be in your path, too.

Install [Trunk](https://trunkrs.dev/) to build and serve the project and make sure
the resulting `trunk` executable is in the PATH for your shell:

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

The pull down the project's source code, change to the dirctory and serve with `trunk`:

```bash
git clone https://github.com/tbogdala/switchboard.git
cd switchboard
trunk serve --release
```
At this point, browse to `http://localhost:8080` on your browser to see the app.


## Notes on Usage

* Firefox, and it's derivatives, may have a small local storage pool by default. You can
  browse to `about:config`, search for the `dom.storage.default_quota` parameter and setting
  it to `102400` for 100MB of local storage.

* Currently, errors are under reported and if something is not acting right, bring up the
  Web Developer Tools (or equivalent) and look at the 'console' tab for error mesasages.


## Publishing to Github Pages

This project keeps a stash of `./dist` in `./docs` for the public facing version
of this project that's hosted on github. It is not generated automatically through CI
but updated manually.

The build is made with an extra flag to `trunk` to adjust for the URL github uses:

```bash
trunk serve --release --public-url /switchboard/
```


## Roadmap

The first major milestone is to get the small version of this project usable. This will entail
just a single endpoint and a simple system message for prompt generation. Chatlog features will
be basic and just include delete, regenerate and edit capabilities. This will provide a minimalistic
starting point for others to use as just a general chat interface.

Aftwards, that version will get branched and run as the 'nano' version of the project.
That's when multiple characters, multiple API endpoints and possibly advanced integrations
with something like an MCP server. Things that are more complicated.


## License

This code is open sourced under the MIT license.
