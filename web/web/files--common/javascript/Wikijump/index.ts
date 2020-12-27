import { Editor } from "./Editor";
import { page } from "./page";
import { render } from "./render";
import { utils } from "./utils";

import { modules } from "@Modules";

const Wikijump = {
  Editor,
  page,
  render,
  utils,

  modules,

  // Stores login-related variables
  vars: {
    rsakey: null as null | string,
    loginSeed: null as null | string,

    // https://github.com/scpwiki/wikijump/pull/78#issuecomment-736901677
    forumThreadId: null as null | number
  }
};

export default Wikijump;
