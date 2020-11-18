

Wikijump.modules.UserAbuseReportModule = {};

Wikijump.modules.UserAbuseReportModule.listeners = {
	sendReport: function(e){
		alert("send?");
	}
}

Wikijump.modules.UserAbuseReportModule.init = function(){
	// limit number of characters
	var l = new  OZONE.forms.lengthLimiter($("abuse-report-text"), $("abuse-report-chcount"), 500);
}

Wikijump.modules.UserAbuseReportModule.init();
