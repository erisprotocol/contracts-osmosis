// const npsUtils = require("nps-utils"); // not required, but handy!

module.exports = {
  scripts: {
    release: {
      default: "bash build_release.sh",
    },
    schema: {
      default: "nps schema.create schema.transform schema.hub",

      create: "bash build_schema.sh",
      transform: "ts-node transform.ts",
      hub: "cd .. && json2ts -i contracts/**/*.json -o ../liquid-staking-scripts/types/update-scaling-factor",
    },
  },
};
