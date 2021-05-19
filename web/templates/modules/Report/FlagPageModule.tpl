<div class="title">{t}Flag page{/t}</div>
<div class="content">
	<h1>Does this page contain objectionable content?</h1>

	<p>
		If you think this page violates
		<a href="{$HTTP_SCHEMA}://{$URL_HOST}/legal:terms-of-service" target="_blank">Terms of Service</a>
		of {$SERVICE_NAME},
		contains objectionable content, may offend etc., you can
		flag this page as objectionable.
	</p>
	<p>
		No content will be automatically removed but the responsible authorities will
		be notified about the page and (if necessary) take some action.
	</p>
	<p>
		Click below to toggle the flag.
	</p>

	<p id="flag-page-options-flag" style="text-align: center; {if isset($flagged)}display: none;{/if} ">
		<span style="color: #4B4; font-size: 150%; border: 1px solid #000; padding: 3px; margin: 5px;">{t}this page is OK{/t}</span>
		<a style="color: #CCC; font-size: 150%; border: 1px solid #DDD; padding: 3px; margin: 5px; text-decoration: none;" href="javascript:;"
		onclick="Wikijump.modules.FlagPageModule.listeners.setFlag(event, true)">{t}this page is objectionable{/t}</a>
	</p>
	<p id="flag-page-options-unflag" style="text-align: center;{if !$flagged}display: none;{/if}">
		<a style="color: #CCC; font-size: 150%; border: 1px solid #DDD; padding: 3px; margin: 5px; text-decoration: none;" href="javascript:;"
		onclick="Wikijump.modules.FlagPageModule.listeners.setFlag(event, false)">{t}this page is OK{/t}</a>
		<span style="color: #B44;font-size: 150%; border: 1px solid #000; padding: 3px; margin: 5px;">{t}this page is objectionable{/t}</span>
	</p>

</div>
<div class="button-bar">
	<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}close this window{/t}</a>
</div>
