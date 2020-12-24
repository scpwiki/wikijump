import { compress } from "compress-tag";

import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

// ok, make some things global here:

const structureArray = {
  '0': "0 (flat/linear)",
  '1': "1",
  '2': "2",
  '3': "3",
  '4': "4",
  '5': "5",
  '6': "6",
  '7': "7",
  '8': "8",
  '9': "9",
  '10': "10",
  '': "forum default",
};

// each entry should be an array with some properties set.
type ForumGroup = {
  name: string;
  description: string;
  visible: boolean;
};
let groups: ForumGroup[];

type ForumCategory = {
  max_nest_level: null | keyof typeof structureArray;
  description: string;
  name: string;
  number_threads?: number;
  category_id?: number;
};
let categories: ForumCategory[][];

const deletedGroups: ForumGroup[] = [];
const deletedCategories: number[] = [];

let defaultNestingLevel: keyof typeof structureArray;

export const ManageSiteForumLayoutModule = {
  listeners: {
    newGroup: function (_event?: Event | null): void {
      // show new group form
      const el = document.getElementById("new-group-window")!;
      const w = new OZONE.dialogs.Dialog();
      w.content = el.innerHTML.replace(/template-id-stub-/g, 'a-');
      w.show();
    },
    editGroup: function (groupIndex: number): void {
      const el = document.getElementById("new-group-window")!;
      const w = new OZONE.dialogs.Dialog();
      w.content = el.innerHTML.replace(/template-id-stub-/g, 'a-');
      w.show();
      const group = groups[groupIndex];
      (<HTMLInputElement>document.getElementById("a-group-name")!).value = group.name;
      (<HTMLInputElement>document.getElementById("a-gindex")!).value = groupIndex.toString();
      (<HTMLTextAreaElement>document.getElementById("a-group-description")!).value = group.description;
    },
    saveGroup: function (_event?: Event | null): void {
      const name = (<HTMLInputElement>document.getElementById("a-group-name")!).value;
      const description = (<HTMLTextAreaElement>document.getElementById("a-group-description")!).value;
      // validate please...
      const errors = [];
      if (name.length == 0) {
        errors[errors.length] = "The name should not be empty";
      }

      if (errors.length > 0) {
        // form HAS errors. print them and exit the function
        document.getElementById("a-form-error-list")!.innerHTML = errors.join('<br/>');
        document.getElementById("a-form-error-container")!.style.display = "block";
      } else {
        // the form is ok, create a new group here...
        const gIndex = (<HTMLInputElement>document.getElementById("a-gindex")!).value;
        const group: ForumGroup = Object.assign({},
          (gIndex == "" || gIndex == null) ? {} : groups[parseInt(gIndex)],
          {
            name,
            description,
            visible: true,
          }
        );
        if (gIndex == "" || gIndex == null) {
          groups[groups.length] = group;
          categories[groups.length - 1] = [];
        } else {
          groups[parseInt(gIndex)] = group;
        }
        ManageSiteForumLayoutModule.utils.refreshDisplay();
        OZONE.dialog.cleanAll();
      }
    },
    hideGroup: function (groupIndex: number): void {
      groups[groupIndex].visible = false;
      ManageSiteForumLayoutModule.utils.refreshDisplay();
    },
    showGroup: function (groupIndex: number): void {
      groups[groupIndex].visible = true;
      ManageSiteForumLayoutModule.utils.refreshDisplay();
    },
    deleteGroup: function (groupIndex: number): void {
      // check if not empty
      if (categories[groupIndex].length > 0) {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = "A non-empty forum group cannot be deleted.";
        w.show();
      } else {
        deletedGroups.push(groups[groupIndex]);
        groups.splice(groupIndex, 1);
        categories.splice(groupIndex, 1);
        ManageSiteForumLayoutModule.utils.refreshDisplay();
      }
    },
    deleteCategory: function (groupIndex: number, categoryIndex: number): void {
      const category = categories[groupIndex][categoryIndex];
      if (category.number_threads && category.number_threads > 0) {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = "A non-empty category cannot be deleted. Consider rather moving the category to a hidden group.";
        w.show();
      } else {
        if (category.category_id) {
          deletedCategories.push(category.category_id);
        }
        categories[groupIndex].splice(categoryIndex, 1);
        ManageSiteForumLayoutModule.utils.refreshDisplay();
      }
    },
    addCategory: function (groupIndex: number): void {
      // show new group form
      const el = document.getElementById("new-category-window")!;
      const w = new OZONE.dialogs.Dialog();
      w.content = el.innerHTML.replace(/template-id-stub-/g, 'a-').replace(/%%ACTION_TYPE%%/, 'Create a new');
      w.show();
      (<HTMLInputElement>document.getElementById("a-group-index")!).value = groupIndex.toString();
    },
    editCategory: function (groupIndex: number, categoryIndex: number): void {
      const cat = categories[groupIndex][categoryIndex];
      const el = document.getElementById("new-category-window")!;
      const w = new OZONE.dialogs.Dialog();
      w.content = el.innerHTML.replace(/template-id-stub-/g, 'a-').replace(/%%ACTION_TYPE%%/, 'Edit');
      w.show();
      (<HTMLInputElement>document.getElementById("a-gcategory-name")!).value = cat.name;
      (<HTMLTextAreaElement>document.getElementById("a-gcategory-description")!).value = cat.description;
      (<HTMLInputElement>document.getElementById("a-category-index")!).value = categoryIndex.toString();
      (<HTMLInputElement>document.getElementById("a-group-index")!).value = groupIndex.toString();
      let mnl = cat.max_nest_level;
      if (mnl == null) {
        mnl = '';
      }
      (<HTMLInputElement>document.getElementById("a-gcategory-structure")!).value = mnl;
    },
    saveCategory: function (_event?: Event | null): void {
      const name = (<HTMLInputElement>document.getElementById("a-gcategory-name")!).value;
      const description = (<HTMLTextAreaElement>document.getElementById("a-gcategory-description")!).value;
      const groupIndex = (<HTMLInputElement>document.getElementById("a-group-index")!).value;
      const maxNestLevel = <keyof typeof structureArray>(<HTMLInputElement>document.getElementById("a-gcategory-structure")!).value;

      // validate please...
      const errors = [];
      if (name.length == 0) {
        errors[errors.length] = "The name should not be empty";
      }

      if (errors.length > 0) {
        // form HAS errors. print them and exit the function
        document.getElementById("a-form-gerror-list")!.innerHTML = errors.join('<br/>');
        document.getElementById("a-form-gerror-container")!.style.display = "block";
      } else {
        // the form is ok, create a new group here...
        const categoryIndex = (<HTMLInputElement>document.getElementById("a-category-index")!).value;
        const category: ForumCategory = Object.assign({},
          (categoryIndex == "" || categoryIndex == null) ?
            {} : categories[parseInt(groupIndex)][parseInt(categoryIndex)],
          {
            name,
            description,
            max_nest_level: maxNestLevel == '' ? null : maxNestLevel
          }
        );

        if (categoryIndex == "" || categoryIndex == null) {
          categories[parseInt(groupIndex)].push(category);
        }
        ManageSiteForumLayoutModule.utils.refreshDisplay();
        OZONE.dialog.cleanAll();
      }
    },
    moveGroupUp: function (groupIndex: number): void {
      if (groupIndex > 0) {
        const tmp1 = groups[groupIndex - 1];
        groups[groupIndex - 1] = groups[groupIndex];
        groups[groupIndex] = tmp1;

        const tmp2 = categories[groupIndex - 1];
        categories[groupIndex - 1] = categories[groupIndex];
        categories[groupIndex] = tmp2;
        ManageSiteForumLayoutModule.utils.refreshDisplay();
      }
    },
    moveGroupDown: function (groupIndex: number): void {
      if (groupIndex < groups.length - 1) {
        const tmp1 = groups[groupIndex + 1];
        groups[groupIndex + 1] = groups[groupIndex];
        groups[groupIndex] = tmp1;

        const tmp2 = categories[groupIndex + 1];
        categories[groupIndex + 1] = categories[groupIndex];
        categories[groupIndex] = tmp2;
        ManageSiteForumLayoutModule.utils.refreshDisplay();
      }
    },
    moveCategoryUp: function (groupIndex: number, categoryIndex: number): void {
      // move within one group or promote to another group...
      // if within group
      if (categoryIndex > 0) {
        const cats = categories[groupIndex];
        const tmp1 = cats[categoryIndex];
        cats[categoryIndex] = cats[categoryIndex - 1];
        cats[categoryIndex - 1] = tmp1;
      } else {
        // inter-group
        if (groupIndex > 0) {
          const cat = categories[groupIndex].shift()!;
          categories[groupIndex - 1].push(cat);
        }
      }
      ManageSiteForumLayoutModule.utils.refreshDisplay();
    },
    moveCategoryDown: function (groupIndex: number, categoryIndex: number): void {
      // move within one group or promote to another group...
      // if within group
      if (categoryIndex < categories[groupIndex].length - 1) {
        const cats = categories[groupIndex];
        const tmp1 = cats[categoryIndex];
        cats[categoryIndex] = cats[categoryIndex + 1];
        cats[categoryIndex + 1] = tmp1;
      } else {
        // inter-group
        if (groupIndex < groups.length - 1) {
          const cat = categories[groupIndex].pop()!;
          categories[groupIndex + 1].splice(0, 0, cat);
        }
      }
      ManageSiteForumLayoutModule.utils.refreshDisplay();
    },
    cancel: function (_event?: Event | null): void {
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-welcome');
      //		OZONE.ajax.requestModule("managesite/ManageSiteModule", {}, ManageSiteForumLayoutModule.callbacks.cancel)
    },
    save: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        action: 'ManageSiteForumAction',
        event: 'saveForumLayout',
        groups: JSON.stringify(groups),
        categories: JSON.stringify(categories),
        deleted_groups: JSON.stringify(deletedGroups),
        deleted_categories: JSON.stringify(deletedCategories)
      };
      OZONE.ajax.requestModule(null, params, ManageSiteForumLayoutModule.callbacks.save);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving forum structure...";
      w.show();
    }
  },
  callbacks: {
    cancel: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      OZONE.utils.setInnerHTMLContent("site-manager", response.body);
    },
    save: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Structure saved.";
      w.show();
      OZONE.ajax.requestModule("managesite/ManageSiteGetForumLayoutModule", {}, ManageSiteForumLayoutModule.callbacks.getLayout);
    },
    getLayout: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      groups = response.groups;
      categories = response.categories;
      defaultNestingLevel = response.defaultNesting;
      ManageSiteForumLayoutModule.utils.refreshDisplay();
    }
  },
  utils: {
    refreshDisplay: function (): void {
      const div = document.getElementById("layout-show-area")!;
      let inner = "";
      for (let i = 0; i < groups.length; i++) {
        const group = groups[i];

        inner += compress`
          <div class="sm-fgroup"
               ${group.visible ? '' : 'style="color: #777"'}>
            <div class="sm-fgroup-name">
              ${OZONE.utils.escapeHtml(group.name)}
              ${group.visible ? '' : '(hidden)' }
            </div>
            <div class="sm-fgroup-description">
              OZONE.utils.escapeHtml(group.description)
            </div>
            <div class="sm-fgroup-options">
              <a href="javascript:;"
                 onclick="ManageSiteForumLayoutModule.listeners.editGroup(${i})">
                edit
              </a>
              |
              <a href="javascript:;"
                 ${
                   group.visible ?
                   `onclick="ManageSiteForumLayoutModule.listeners.showGroup(${i})">show` :
                   `onclick="ManageSiteForumLayoutModule.listeners.hideGroup(${i})">hide`
                 }
              </a>
              |
              <a href="javascript:;"
                 onclick="ManageSiteForumLayoutModule.listeners.deleteGroup(${i})">
                delete
              </a>
              |
              <a href="javascript:;"
                 onclick="ManageSiteForumLayoutModule.listeners.addCategory(${i})">
                add category
              </a>
              |
              <a href="javascript:;"
                 onclick="ManageSiteForumLayoutModule.listeners.moveGroupUp(${i})">
                move up
              </a>
              |
              <a href="javascript:;"
                 onclick="ManageSiteForumLayoutModule.listeners.moveGroupDown(${i})">
                move down
              </a>
            </div>
        `;

        // now add all categories...
        const cats = categories[i];
        for (let j = 0; j < cats.length; j++) {
          const cat = cats[j];
          if (!cat.number_threads) {
            cat.number_threads = 0;
          }
          inner += compress`
            <div class="sm-fcat">
              <div class="sm-fcat-name">
                ${OZONE.utils.escapeHtml(cat.name)}
              </div>
              <div class="sm-fcat-description">
                ${OZONE.utils.escapeHtml(cat.description)}
              </div>
              <div class="sm-fcat-info">
                number of threads: ${cat.number_threads}<br/>
                maximum nesting level: ${
                  cat.max_nest_level == null ?
                    `default forum nesting  (${structureArray[defaultNestingLevel]})` :
                    structureArray[cat.max_nest_level]
                }
                <br/>
              </div>
              <div class="sm-fcat-options">
                <a href="javascript:;"
                   onclick="ManageSiteForumLayoutModule.listeners.editCategory(${i}, ${j})">
                  edit
                </a>
                |
                <a href="javascript:;"
                   onclick="ManageSiteForumLayoutModule.listeners.deleteCategory(${i}, ${j})">
                  delete
                </a>
                |
                <a href="javascript:;"
                   onclick="ManageSiteForumLayoutModule.listeners.moveCategoryUp(${i}, ${j})">
                  move up
                </a>
                |
                <a href="javascript:;"
                   onclick="ManageSiteForumLayoutModule.listeners.moveCategoryDown(${i}, ${j})">
                  move down
                </a>
              </div>
            </div>
          `;
        }
        inner += '</div>'; // close sm-fgroup div
      }
      div.innerHTML = inner;
    }
  },
  init: function (): void {
    YAHOO.util.Event.addListener("new-group-b", "click", ManageSiteForumLayoutModule.listeners.newGroup);

    // get layout:

    OZONE.ajax.requestModule("managesite/ManageSiteGetForumLayoutModule", {}, ManageSiteForumLayoutModule.callbacks.getLayout);
  }
};

ManageSiteForumLayoutModule.init();
