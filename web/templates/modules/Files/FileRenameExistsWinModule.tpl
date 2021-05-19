<div class="content">
<h1>{t}File exists{/t}</h1>

<table class="form">
	<tr>
		<td>
			{t}Current name{/t}:
		</td>
		<td>
			{$file->getFilename()|escape}
		</td>
	</tr>
	<tr>
		<td>
			{t}New name{/t}:
		</td>
		<td>
			{$newFile->getFilename()|escape}
		</td>
	</tr>
</table>

{if isset($hasPermission)}
<p>
	{t}Unfortunately the file named{/t} {$newFile->getFilename()|escape} {t}already exists{/t}. {t}Do you want
	to overwrite it?{/t}
</p>
<input type="hidden" id="file-rename-name" value="{$newFile->getFilename()|escape}"/>
</div>
<div class="button-bar">
	<a href="javascript:;" onclick="Wikijump.modules.PageFilesModule.listeners.renameFile2(event, true)">{t}overwrite{/t}</a>
	<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}cancel{/t}</a>
</div>
{else}
<p>
	{t}Unfortunately the file named{/t} {$newFile->getFilename()|escape} {t}already exists{/t}. {t}You have no
	permission to overwrite/delete files on this page.{/t}
</p>
</div>
<div class="button-bar">
	<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}close message{/t}</a>
</div>
{/if}

