
<div class="new-page-box" style="text-align: center; margin: 1em 0">
	<form action="dummy.html" method="get" onsubmit="Wikijump.modules.NewPageHelperModule.listeners.create(event)">
		<input class="text" name="pageName" type="text" size="{$size|escape}" maxlength="60" style="margin: 1px"/>{if isset($templates)}
		<select name="template" style="margin: 1px">
			<option value="" selected="selected">-- {t}Select a template{/t} --</option>
			{foreach from=$templates item=template}
				<option value="{$template->getPageId()}">{$template->getTitleOrUnixName()|escape}</option>
			{/foreach}
		</select>
		{/if}<input type="submit" class="button" value="{if isset($button)}{$button|escape}{else}{t}create page{/t}{/if}" style="margin: 1px"/>
		{if isset($categoryName)}
			<input type="hidden" name="categoryName" value="{$categoryName}"/>
		{/if}
		{if isset($template)}
			<input type="hidden" name="template" value="{$template->getPageId()}"/>
		{/if}
		{if isset($format)}
			<input type="hidden" name="format" value="{$format|escape}"/>
		{/if}
		{if isset($autoincrement)}
			<input type="hidden" name="autoincrement" value="true"/>
		{/if}

	</form>
</div>
{if isset($formatError)}
	<div class="error-block">
		The format {$formatError|escape} is not a valid regular expression in the NewPage module above.
	</div>
{/if}
