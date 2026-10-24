// Eseguito in fase di build: salva i nodi in data/flowcharts.json
const fs = require("fs");
const path = require("path");
const { scrapeAll } = require("../scraper");
scrapeAll(m => console.log(m)).then(d => {
  const n = Object.keys(d.chapters).length;
  fs.writeFileSync(path.join(__dirname, "..", "data", "flowcharts.json"), JSON.stringify(d));
  console.log(`Salvati ${n} capitoli`);
  if (n === 0) process.exitCode = 1;
});
