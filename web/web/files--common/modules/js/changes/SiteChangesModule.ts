import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { Wikirequest } from "wikirequest";
declare const WIKIREQUEST: Wikirequest;
declare type YahooResponse = any;


export const SiteChangesModule = {
  listeners: {
    updateList: function(pageNo: number | null): void {
      const params: RequestModuleParameters = {
        perpage: (<HTMLSelectElement>document.getElementById("rev-perpage")!).value,
        pageId: WIKIREQUEST.info.pageId,
        categoryId: (<HTMLSelectElement>document.getElementById("rev-category")!).value,
      };
      if(pageNo != null){
        params.page = pageNo;
      }else{
        params.page = 1;
      }
      // which revisions...
      const options: Record<string, boolean> = {};
      if((<HTMLInputElement>document.getElementById("rev-type-all")!).checked) {
        options.all = true;
      }
      if((<HTMLInputElement>document.getElementById("rev-type-source")!).checked) {
        options.source = true;
      }
      if((<HTMLInputElement>document.getElementById("rev-type-title")!).checked) {
        options.title = true;
      }
      if((<HTMLInputElement>document.getElementById("rev-type-move")!).checked) {
        options.move = true;
      }
      if((<HTMLInputElement>document.getElementById("rev-type-files")!).checked) {
        options.files = true;
      }
      if((<HTMLInputElement>document.getElementById("rev-type-new")!).checked) {
        options['new'] = true;
      }
      if((<HTMLInputElement>document.getElementById("rev-type-meta")!).checked) {
        options.meta = true;
      }

      params.options = JSON.stringify(options);
      //Wikijump.modules.PageHistoryModule.vars.params = params; // for pagination

      OZONE.ajax.requestModule("changes/SiteChangesListModule", params, SiteChangesModule.callbacks.updateList);
    }
  },

  callbacks: {
    updateList: function(response: YahooResponse): void {
      if(!Wikijump.utils.handleError(response)) {return;}

      document.getElementById("site-changes-list")!.innerHTML = response.body;
      OZONE.utils.formatDates("site-changes-list");
      OZONE.dialog.hovertip.makeTip(
        document.getElementById("site-changes-list")!.getElementsByTagName('span'),
        { style: { width: 'auto' } }
      );
    }

  },

  init: function (): void {
    OZONE.dom.onDomReady(function(){
      OZONE.utils.formatDates("site-changes-list");
      OZONE.dialog.hovertip.makeTip(document.getElementById("site-changes-list")!.getElementsByTagName('span'),
                                    {style: {width: 'auto'}});
    }, "dummy-ondomready-block");
  }
};

SiteChangesModule.init();
