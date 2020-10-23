import { ajax } from "./ajax";
import { dialog } from "./dialog";
import { dialogs } from "./dialogs";
import { dom } from "./dom";
import { forms } from "./forms";
import { loc } from "./loc";
import { utils } from "./utils";
import { visuals } from "./visuals";

const OZONE = {
  ajax,
  dialog,
  dialogs,
  dom,
  forms,
  loc,
  utils,
  visuals,

  lang: 'en', // default language
  request: {},
  init: {} // This was an empty function
};

export default OZONE;
