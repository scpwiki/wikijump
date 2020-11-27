import { compress } from "compress-tag";

import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

export const APAvatarModule = {
  vars: {
    im48: null as null | string,
    im16: null as null | string
  },
  listeners: {
    startUpload: function (_event?: Event | null): void {
      document.getElementById("upload-wait")!.style.display = "block";
    },
    uploaded: function (status: string, im48: string, im16: string): void {
      if (status != "ok") {
        const er = new OZONE.dialogs.ErrorDialog();
        er.content = compress`
          The uploaded file cannot be used as a buddy icon.<br/>
          Please upload a valid .png, .jpg or .gif image.
        `;
        er.show();
        //			alert("The uploaded file cannot be used as a buddy icon.\n" +
        return;
      }
      const path = '/common--tmp/avatars-upload/';
      (<HTMLImageElement>document.getElementById("avatar-preview-large")!).src = `${path}${im48}`;
      (<HTMLImageElement>document.getElementById("avatar-preview-small")!).src = `${path}${im16}`;

      APAvatarModule.vars.im16 = im16;
      APAvatarModule.vars.im48 = im48;
      document.getElementById("avatar-preview")!.style.display = "block";
      document.getElementById('file-upload-div')!.style.display = 'none';
      document.getElementById("uri-upload-div")!.style.display = 'none';
      document.getElementById("upload-wait")!.style.display = "none";
    },
    useIt: function (_event?: Event | null): void {
      // sets the avatar permanently
      if (!APAvatarModule.vars.im16) { return; }
      const params: RequestModuleParameters = {
        im48: APAvatarModule.vars.im48,
        im16: APAvatarModule.vars.im16,
        action: 'AccountProfileAction',
        event: "setAvatar"
      };
      OZONE.ajax.requestModule("Empty", params, APAvatarModule.callbacks.useIt);
    },
    reset: function (event: Event): void {
      document.getElementById('avatar-choice1')!.style.display = '';
      document.getElementById('file-upload-div')!.style.display = 'none';
      document.getElementById("uri-upload-div")!.style.display = 'none';
      document.getElementById("avatar-preview")!.style.display = 'none';
      document.getElementById("upload-wait")!.style.display = "none";
      APAvatarModule.vars.im16 = null;
      APAvatarModule.vars.im48 = null;
      YAHOO.util.Event.stopEvent(event);
    },
    deleteAvatar: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        action: 'AccountProfileAction',
        event: "deleteAvatar"
      };
      OZONE.ajax.requestModule("Empty", params, APAvatarModule.callbacks.deleteAvatar);
    },
    uploadUri: function (_event?: Event | null): void {
      const uri = (<HTMLInputElement>document.getElementById("upload-uri")!).value;
      if (!uri.match(/^(http[s]?:\/\/)|(ftp:\/\/)[a-zA-Z0-9-]+\/.*/)) {
        const er = new OZONE.dialogs.ErrorDialog();
        er.content = "This is not a valid URI address.";
        er.show();
        return;
      }
      document.getElementById("upload-wait")!.style.display = "block";
      const params: RequestModuleParameters = {
        action: 'AccountProfileAction',
        event: "uploadAvatarUri",
        uri: uri
      };
      OZONE.ajax.requestModule("Empty", params, APAvatarModule.callbacks.uploadUri);
    }
  },
  callbacks: {
    useIt: function (_response: YahooResponse): void {
      /*
         document.getElementById('file-upload-div')!.style.display='none';
         document.getElementById("uri-upload-div")!.style.display='none';
         document.getElementById("avatar-preview")!.style.display='none';
         document.getElementById("avatar-success")!.style.display='block';
         APAvatarModule.vars.im16 = null;
         APAvatarModule.vars.im48 = null;
       */
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Your buddy icon has been changed!";
      w.show();
      setTimeout(() => OZONE.ajax.requestModule("account/profile/APAvatarModule", {}, Wikijump.modules.AccountModule.callbacks.menuClick), 1500);
    },
    deleteAvatar: function (_response: YahooResponse): void {
      // simply reload this module.
      OZONE.ajax.requestModule('account/profile/APAvatarModule', {}, Wikijump.modules.AccountModule.callbacks.menuClick);
    },
    uploadUri: function (response: YahooResponse): void {
      if (response.status != "ok") {
        const er = new OZONE.dialogs.ErrorDialog();
        er.content = "This image cannot be used as your buddy icon. (" + response.status + ")";
        er.show();
        return;
      }
      APAvatarModule.listeners.uploaded(response.status, response.im48, response.im16);
    }
  }
};
