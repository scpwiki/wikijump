import { Wikirequest } from "wikirequest";
declare const WIKIREQUEST: Wikirequest;
declare const YAHOO: any;

declare const HTTP_SCHEMA: 'http' | 'https';
declare const URL_HOST: string;

function logininit (): void {
  const sIfr = (<HTMLIFrameElement>document.getElementById("login-iframe")!);
  if (!sIfr) {
    setTimeout(() => logininit(), 500);
    return;
  }

  let url = `${HTTP_SCHEMA}://${URL_HOST}/default_flow.php?login__LoginIframeScreen`;
  if (YAHOO.env.ua.ie > 0) {
    url = '/default_flow.php?login__LoginIframeScreen';
  }
  url += '/siteId/' + WIKIREQUEST.info.siteId;
  url += '/categoryId/' + WIKIREQUEST.info.categoryId;
  url += '/themeId/' + WIKIREQUEST.info.themeId;
  url += '/url/' + encodeURIComponent(window.location.href);

  sIfr.src = url;
}

logininit();
