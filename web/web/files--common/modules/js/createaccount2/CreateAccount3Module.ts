import OZONE from "@/javascript/OZONE";

OZONE.dom.onDomReady(function (): void {
  // change links to http://...
  const els = document.getElementsByTagName('a');
  for (let i = 0; i < els.length; i++) {
    els[i].href = els[i].href.replace(/^https/, 'http');
  }
}, "dummy-ondomready-block");
