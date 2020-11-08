import OZONE from "@/javascript/OZONE";
import { ogettext } from "@/javascript/OZONE/loc";

export const utils = {
  handleError: function (
    response: { status: string, message: string }
  ): boolean {
    /**
     * Detects whether or not an AJAX response contained an error and, if there
     * was, reports it back to the user in a dialog.
     *
     * @param response: The AJAX response.
     * @returns Boolean indicating whether or not there was an error.
     */
    // Response is defined in AjaxModuleWikiFlowController.php
    if (response.status !== 'ok') {
      const w = new OZONE.dialogs.ErrorDialog();
      if (response.status === 'no_permission') {
        w.title = ogettext('Permission error');
      }
      w.content = '<h1>' + ogettext('Oooops!') + '</h1><p>' + response.message + '</p>';
      w.show();
      return false;
    } else {
      return true;
    }
  }
};
