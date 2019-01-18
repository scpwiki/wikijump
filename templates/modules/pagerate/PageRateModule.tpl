<h1>{t}Page rating{/t}</h1>

<p>
	{t}Simply rate contents of this page.{/t}
</p>

<div style="text-align: center">
	{module name="pagerate/PageRateWidgetModule" pageId=$pageId}
</div>

{if $visibility == 'v'}
	<p>
		<a href="javascript:;" onclick="WIKIDOT.modules.PageRateModule.listeners.showWho(event, {$pageId})">{t}look who rated this page{/t}</a>
	</p>
	
	<div id="who-rated-page-area"></div>
{/if}