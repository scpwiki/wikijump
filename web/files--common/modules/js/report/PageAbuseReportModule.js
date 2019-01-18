/*
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * Code licensed under the GNU Affero General Public 
 * License version 3 or later.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 */

WIKIDOT.modules.PageAbuseReportModule = {};

WIKIDOT.modules.PageAbuseReportModule.listeners = {
	sendReport: function(e){
		alert("send?");
	}
}

WIKIDOT.modules.PageAbuseReportModule.init = function(){
	// limit number of characters
	var l = new  OZONE.forms.lengthLimiter($("abuse-report-text"), $("abuse-report-chcount"), 500);
}

WIKIDOT.modules.PageAbuseReportModule.init();
