<div class="content">
<h1>{t}File exists{/t}</h1>

<table class="form">
	<tr>
		<td>
			{t}File name{/t}:
		</td>
		<td>
			<strong>{$file->getFilename()|escape}</strong>
		</td>
	</tr>
	<tr>
		<td>
			{t}Current page{/t}:
		</td>
		<td>
			{$page->getUnixName()|escape}
		</td>
	</tr>
	<tr>
		<td>
			{t}Destination page{/t}:
		</td>
		<td>
			{$destinationPage->getUnixName()|escape}
		</td>
	</tr>
</table>

{if isset($hasPermission)}
	<p>
		{t}Unfortunately the file named{/t} {$file->getFilename()|escape} {t}already exists as an attachment
		to page{/t} {$destinationPage->getUnixName()|escape}. {t}Do you want to overwrite it?{/t}
	</p>
	<input type="hidden" id="file-move-page" value="{$destinationPage->getUnixName()|escape}"/>
	</div>
	<div class="button-bar">
		<a href="javascript:;" onclick="Wikijump.modules.PageFilesModule.listeners.moveFile2(event, true)">{t}overwrite{/t}</a>
		<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}cancel{/t}</a>
	</div>
{else}
	<p>
		{t}Unfortunately the file named{/t} {$file->getFilename()|escape} {t}already exists as an attachment
		to page{/t} {$destinationPage->getUnixName()|escape}. {t}You have no
		permission to overwrite/delete files on this page.{/t}
	</p>
	</div>
	<div class="button-bar">
		<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}close message{/t}</a>
	</div>
{/if}
