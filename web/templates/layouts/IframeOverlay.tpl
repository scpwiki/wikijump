<!DOCTYPE html
     PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN"
     "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="{$site->getLanguage()}" lang="{$site->getLanguage()}">

<head>
 	<title>{$site->getName()}{if $wikiPage && $wikiPage->getTitle()}: {$wikiPage->getTitle()|escape}{/if}</title>
 	<script type="text/javascript" src="/common--javascript/json.js"></script>
	<script type="text/javascript" src="/common--javascript/combined.js"></script>
 	<script type="text/javascript" src="/common--dist/bundle.js"></script>

 	<script  type="text/javascript">
 		// global request information
 		{literal}
 		var WIKIREQUEST = {};
 		WIKIREQUEST.info = {};
 		{/literal}
 		WIKIREQUEST.info.domain = "{$site->getDomain()}";
 		WIKIREQUEST.info.siteId = {$site->getSiteId()};
 		OZONE.request.date = new Date();
 		WIKIREQUEST.info.lang = '{$site->getLanguage()}';
 		WIKIREQUEST.info.lang = "{$site->getLanguage()}";
 		OZONE.lang = "{$site->getLanguage()}";
// 		window.onload = WikijumpInit();
 	</script>

 	<meta http-equiv="content-type" content="text/html;charset=UTF-8"/>
    <meta http-equiv="content-language" content="{$site->getLanguage()}"/>


 	<script type="text/javascript" src="/common--javascript/crypto/rsa.js"></script>

 	{if isset($useCustomDomainScript)}
 		{module name="Login/CustomDomainScriptModule"}
 	{/if}
   	<style type="text/css" id="internal-style">

   		{foreach from=$theme->getStyleUrls() item=file}
   			@import url({$file});
   		{/foreach}

    </style>

    <link rel="shortcut icon" href="/common--theme/base/images/favicon.gif"/>
    <link rel="icon" type="image/png" href="/common--theme/base/images/favicon.gif"/>

</head>

<body id="html-body" style="background-color:transparent;min-width:0px;background-image: none;">

	{$screen_placeholder}
	<div style="display:none" id="dummy-ondomready-block"></div>

</body>

</html>
