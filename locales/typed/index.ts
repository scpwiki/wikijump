import * as fs from 'fs';
import { FluentResource } from "@fluent/bundle";

const input = process.argv[2] || "../fluent";
const output = process.argv[3] || "types";

if (!fs.existsSync(output)) {
  fs.mkdirSync(output, { recursive: true });
}

let combined: { [key: string]: string } = {};

fs.readdirSync(input).forEach((folder) => {
  const data = fs.readdirSync(`${input}/${folder}`).map((file) => {
    if (!file.includes("en")) return;

    const files = fs.readFileSync(`${input}/${folder}/${file}`, "utf8");

    return new FluentResource(files).body.map((entry) => {
      return entry.id;
    });
  });

  const filterd = data.filter((entry) => entry !== undefined);

  const flat = filterd.flat();

  const unique = [...new Set(flat)];

  unique.forEach((entry) => {
    combined["readonly " + entry] = "string";
  });
});

const types = JSON.stringify(combined, null, 2)
  .replace(/"readonly /g, 'readonly "')
  .replace(/"/g, "")
  .replace(/,/g, ";")
  .replace(/(?=\S*['-])([a-zA-Z0-9'-]+)/g, '"$1"');

fs.writeFileSync(`${output}/index.ts`, `export interface Locales ${types}`);
