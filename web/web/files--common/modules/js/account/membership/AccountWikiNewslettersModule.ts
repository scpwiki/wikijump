declare const YAHOO: any;

export const AccountWikiNewslettersModule = {
  listeners: {
    checkAll: function(_event: Event | null, value: boolean): void {
      const inps = YAHOO.util.Dom.getElementsByClassName("receive-newsletter", "input", "receive-wiki-newsletters-form");
      for(let i=0; i<inps.length; i++){
        inps[i].checked = value;
      }
    }
  }
}
