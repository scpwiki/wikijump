[<< Return to the README](README.md)

## ftml-http Routes

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
| `POST` | `/parse` | Text | `ParseOutcome<SyntaxTree>` | Runs the parser on the input string and returns the abstract syntax tree. |
| `POST` | `/parse/only` | Text | `ParseOutcome<SyntaxTree>` | Same as above, but the preprocessor is not run first. |
| `POST` | `/render/html` | Text | `ParseOutcome<HtmlOutput>` | Performs the full rendering process, from preprocessing, tokenization, parsing, and then rendering. |
| `POST` | `/render/html/only` | Text | `ParseOutcome<HtmlOutput>` | Same as above, but the preprocessor is not run first. |
| `POST` | `/render/debug` | Text | `ParseOutcome<String>` | Performs rendering, as above, but uses `ftml::DebugRender`. |
| `POST` | `/render/debug/only` | Text | `ParseOutcome<String>` | Same as above, but the preprocessor is not run first. |
