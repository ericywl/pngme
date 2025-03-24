# PNGME

PNGME is a project idea taken from https://jrdngr.github.io/pngme_book/introduction.html.

The code is written in Rust.

## Commands

There are 4 commands available:

#### Encode

Encodes a secret message into a chunk of specified chunk type, appended to the PNG file given. An output filename can be passed optionally to avoid overwritting the input file.

```
cargo run -- encode <input.png> --chunk_type <chunk_type_str> --message <some_secret_message> --output <optional_output.png>
```

#### Decode

Decodes a chunk of specified chunk type from the PNG. If no message is found, it will simply say `No message found`. Otherwise, it will print out the secret message.

```
cargo run -- decode <input.png> --chunk_type <chunk_type_str>
```

#### Remove

Removes a chunk of specified chunk type from the PNG. If no message is found, it will return an error. Otherwise, it will print out the chunk that has been removed.

```
cargo run -- remove <input.png> --chunk_type <chunk_type_str>
```

#### Print

Prints all chunks from the PNG.

```
cargo run -- print <input.png>
```

## Chunk layout

Read more about chunk layout here: http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html.

The chunk type passed into the commands above must adhere to the specification.
