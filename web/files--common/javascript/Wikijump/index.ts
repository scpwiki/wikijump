import { Editor } from "./Editor";
import { page } from "./page";
import { render } from "./render";
import { utils } from "./utils";

const Wikijump = {
  Editor,
  page,
  render,
  utils,

  // Module store: when a module is loaded it will be registered to this
  // object
  modules: {},

  // Stores login-related variables
  vars: {}
};

export default Wikijump;
