const fs = require("fs")
const path = require("path")

const EXCLUDE = [/fail/, /error/, /iframe/, /html/, /image/]

const testDir = path.join(__dirname, "../../../", "ftml/test")
const jsons = fs
  .readdirSync(testDir)
  .filter(file => file.endsWith(".json"))
  .filter(file => !EXCLUDE.some(re => re.test(file)))

str = ""

for (const file of jsons) {
  const json = require(path.join(testDir, file))
  const name = file.replace(".json", "")
  str += `[!-- ${name} --]\n${json.input}\n\n`
}

fs.writeFileSync(path.resolve(__dirname, "./ftml-test-cases-export.ftml"), str)
