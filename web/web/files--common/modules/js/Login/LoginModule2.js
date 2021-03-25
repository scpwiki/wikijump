

function logininit(){
	var sIfr = $("login-iframe");
	if(!sIfr){
		setTimeout("logininit()", 500);
		return;
	}

	var url=window.location.protocol+'//'+URL_HOST+'/default_flow.php?Login__LoginIframeScreen';
	if(YAHOO.env.ua.ie > 0){
		url = '/default_flow.php?Login__LoginIframeScreen';
	}
	url += '/siteId/'+WIKIREQUEST.info.siteId;
	url += '/categoryId/'+WIKIREQUEST.info.categoryId;
	url += '/themeId/'+WIKIREQUEST.info.themeId;
	url += '/url/'+ encodeURIComponent(window.location.href);

	sIfr.src=url;
}

logininit();
