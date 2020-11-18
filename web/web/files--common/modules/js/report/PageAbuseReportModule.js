

Wikijump.modules.PageAbuseReportModule = {};

Wikijump.modules.PageAbuseReportModule.listeners = {
	sendReport: function(e){
		alert("send?");
	}
}

Wikijump.modules.PageAbuseReportModule.init = function(){
	// limit number of characters
	var l = new  OZONE.forms.lengthLimiter($("abuse-report-text"), $("abuse-report-chcount"), 500);
}

Wikijump.modules.PageAbuseReportModule.init();
