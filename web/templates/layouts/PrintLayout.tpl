<!DOCTYPE html
     PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN"
     "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="{$site->getLanguage()}" lang="{$site->getLanguage()}">

<head>
 	<title>{$site->getName()}{if $wikiPage && $wikiPage->getTitle()}: {$wikiPage->getTitle()|escape}{/if}</title>
 	<script type="text/javascript" src="/common--javascript/json.js"></script>

 	<script type="text/javascript" src="/common--javascript/yahooui/yahoo-min.js"></script>
 	<script type="text/javascript" src="/common--javascript/yahooui/connection-min.js"></script>
 	<script type="text/javascript" src="/common--javascript/yahooui/event-min.js"></script>
 	<script type="text/javascript" src="/common--javascript/yahooui/dom-min.js"></script>
 	<script type="text/javascript" src="/common--javascript/yahooui/dragdrop-min.js"></script>
 	<script type="text/javascript" src="/common--javascript/yahooui/autocomplete-min.js"></script>

 	<script type="text/javascript" src="/common--javascript/moofx/prototype.lite.js"></script>
 	<script type="text/javascript" src="/common--javascript/moofx/moo.fx.js"></script>
	<script type="text/javascript" src="/common--javascript/moofx/moo.fx.pack.js"></script>
 	<script type="text/javascript" src="/common--dist/bundle.js"></script>

 	<script  type="text/javascript">
 		// global request information
 		{literal}
 		var WIKIREQUEST = {};
 		WIKIREQUEST.info = {};
 		{/literal}
 		WIKIREQUEST.info.domain = "{$site->getDomain()}";
 		WIKIREQUEST.info.siteId = {$site->getSiteId()};
 		WIKIREQUEST.info.requestPageName = "{$wikiPageName}";
 		OZONE.request.timestamp = %%%CURRENT_TIMESTAMP%%%;
 		OZONE.request.date = new Date();
 		WIKIREQUEST.info.lang = '{$site->getLanguage()}';
 		{if $wikiPage}
 		WIKIREQUEST.info.pageUnixName = "{$wikiPage->getUnixName()}";
 		WIKIREQUEST.info.pageId = {$wikiPage->getPageId()};
 		{/if}
// 		window.onload = WikijumpInit();
 	</script>

 	<meta http-equiv="content-type" content="text/html;charset=UTF-8"/>
    <meta http-equiv="content-language" content="{$site->getLanguage()}"/>


 	<script type="text/javascript" src="/common--javascript/printview.js"></script>

   	<style type="text/css" id="internal-style">

   		{foreach from=$theme->getStyleUrls() item=file}
   			@import url({$file});
   		{/foreach}
   		@import url(/common--theme/base/css/print.css?0);

    </style>
    <style type="text/css" media="print">
	    @import url(/common--theme/base/css/print2.css?0);
    </style>
</head>

  <body id="html-body">

	<div id="container">
		<div id="print-options">
			<table>
				<tr>
					<td>
						{t}Base font size{/t}:
					</td>
					<td>
						<a href="javascript:;" onclick="Wikijump.printview.listeners.changeFontSize(event, '6pt')">6pt</a>
						| <a href="javascript:;" onclick="Wikijump.printview.listeners.changeFontSize(event, '8pt')">8pt</a>
						| <a href="javascript:;" onclick="Wikijump.printview.listeners.changeFontSize(event, '10pt')">10pt</a>
						| <a href="javascript:;" onclick="Wikijump.printview.listeners.changeFontSize(event, '12pt')">12pt</a>
						| <a href="javascript:;" onclick="Wikijump.printview.listeners.changeFontSize(event, '14pt')">14pt</a>
						| <a href="javascript:;" onclick="Wikijump.printview.listeners.changeFontSize(event, '16pt')">16pt</a>
					</td>
				</tr>
				<tr>
					<td>
						{t}Body font{/t}:
					</td>
					<td>
						<a href="javascript:;" onclick="Wikijump.printview.listeners.changeFontFamily(event,'original')">{t}original{/t}</a>
						| <a href="javascript:;" onclick="Wikijump.printview.listeners.changeFontFamily(event,'Georgia')">Georgia</a>
						| <a href="javascript:;" onclick="Wikijump.printview.listeners.changeFontFamily(event,'roman')">Times New Roman</a>
						| <a href="javascript:;" onclick="Wikijump.printview.listeners.changeFontFamily(event,'Serif')">Serif (generic)</a>
						| <a href="javascript:;" onclick="Wikijump.printview.listeners.changeFontFamily(event,'Arial, Helvetica')">Arial/Helvetica</a>
					</td>
				</tr>
				<tr>
					<td>
						{t}Source info{/t}:
					</td>
					<td>
						<a href="javascript:;" onclick="Wikijump.printview.listeners.toggleSourceInfo(event)">{t}toggle visibility{/t}</a>
					</td>
				</tr>
				<tr>
					<td>{t}Options{/t}:</td>
					<td>
						<b><a href="javascript:;" onclick="window.print()">{t}PRINT THE PAGE{/t}</a></b>
						| <a href="javascript:;" onclick="window.close()">{t}close this window{/t}</a>
					</td>
				</tr>
			</table>
		</div>

	  	<div id="print-head">
		  	{t}Site{/t}: <b>{$site->getName()|escape}</b> at {$HTTP_SCHEMA}://{$site->getDomain()}
		  	<br/>
		  	{t}Source page{/t}: <b>{$wikiPage->getTitleOrUnixName()|escape}</b> at {$HTTP_SCHEMA}://{$site->getDomain()}/{$wikiPage->getUnixName()}
	  	</div>
		<div id="content-wrap">
			<div id="main-content">
				<div id="action-area-top"></div>
				{if $wikiPage->getTitle() != ''}
				<div id="page-title">
					{$wikiPage->getTitle()|escape}
				</div>
				{/if}

				<div id="page-content">
					{$screen_placeholder}
				</div>

				<div id="page-info" >
					{t}page revision{/t}: {$wikiPage->getRevisionNumber()}, {t}last edited{/t}: <span class="odate">{$wikiPage->getDateLastEdited()->getTimestamp()}|%e %b %Y, %H:%M %Z (%O ago)</span>
				</div>
				<div id="action-area" style="display: none"></div>
			</div>
		</div>

 		<hr/>

 		<div id="license-area" class="license-area">
 			{$licenseHtml|replace:'%%UNLESS%%':'Unless stated otherwise Content of this page is licensed under'}
		</div>

		<hr/>

		<div id="print-footer">
			Part of {$SERVICE_NAME|escape} &#8212; Powered by Wikijump
		</div>
 	</div>

 	<div style="display:none" id="dummy-ondomready-block"></div>

  </body>

</html>
