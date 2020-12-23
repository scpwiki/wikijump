[<< Return to the README](README.md)

## ftml-server Routes

Note that input text are really simple JSON objects in the following form:
```json
{
    "text": "<your input string>"
}
```

The currently available API routes in the server are:

| Method | Route | Input | Output | Description |
|--------|-------|-------|--------|-------------|
| Any | `/ping` | None | `String` | See if you're able to connect to the server. |
| Any | `/version` | None | `String` | Outputs what version of ftml is being run. |
| `POST` | `/preprocess` | Text | `String` | Runs the preprocessor on the given input string. |
| `POST` | `/tokenize` | Text | `Vec<ExtractedToken>` | Runs the tokenizer on the input string and returns the extracted tokens. |
| `POST` | `/tokenize/only` | Text | `Vec<ExtractedToken>` | Same as above, but the preprocessor is not run first. |
| `POST` | `/parse` | Text | `ParseResult<SyntaxTree>` | Runs the parser on the input string and returns the abstract syntax tree. |
| `POST` | `/parse/only` | Text | `ParseResult<SyntaxTree>` | Same as above, but the preprocessor is not run first. |
| `POST` | `/render/html` | Text | `ParseResult<HtmlOutput>` | Performs the full rendering process, from preprocessing, tokenization, parsing, and then rendering. |
| `POST` | `/render/html/only` | Text | `ParseResult<HtmlOutput>` | Same as above, but the preprocessor is not run first. |
| `POST` | `/render/debug` | Text | `ParseResult<String>` | Performs rendering, as above, but uses `ftml::DebugRender`. |
| `POST` | `/render/debug/only` | Text | `ParseResult<String>` | Same as above, but the preprocessor is not run first. |
