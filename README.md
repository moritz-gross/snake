### An implementation of the videogame *Snake* using Rust

My first pet project using this language.



![gameplay](https://github.com/user-attachments/assets/f77945dd-ae86-4ad7-907b-4c222efeb7c5)


## Run

```
cargo run
```

## Debug overlays

```
cargo run --features debug_draw
```

## Spectator mode (local WebSocket + HTTP)

```
cargo run --features spectator
```

Then open:

- On the same machine: `http://127.0.0.1:8000/`
- On another device in the same Wi-Fi: `http://<your-mac-ip>:8000/`

The WebSocket server runs on port `9001`. The spectator page connects automatically.


