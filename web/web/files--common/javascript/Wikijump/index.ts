import { Editor } from "./Editor";
import { page } from "./page";
import { render } from "./render";
import { utils } from "./utils";

import { modules } from "@Modules"

const Wikijump = {
  Editor,
  page,
  render,
  utils,

  modules,

  // Stores login-related variables
  vars: {
    rsakey: null as unknown,
    loginSeed: null as unknown
  }
};

export default Wikijump;
