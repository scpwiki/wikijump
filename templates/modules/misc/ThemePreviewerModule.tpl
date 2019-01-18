{if !$noUi}
<div style="text-align: center">
	<form action="about:blank">
		<select class="select" id="themePreviewerSelect"
			onchange="window.location.href=window.location.href.replace(/\?.*$/, '').replace(/(\/theme_id\/[0-9]+)|$/, '/theme_id/'+$('themePreviewerSelect').value);">
			{foreach from=$themes item=theme}
				<option {if $theme->getThemeId()==$currentTheme->getThemeId()}selected="selected"{/if} value="{$theme->getThemeId()}">{$theme->getName()|escape}</option>
			{/foreach}
		</select>
		
		{*
		<input type="button" class="button" value="change theme" onclick="window.location.href=window.location.href.replace(/(\/theme_id\/[0-9]+)|$/, '/theme_id/'+$('themePreviewerSelect').value);"/>
		*}
		
	</form>
</div>
{/if}