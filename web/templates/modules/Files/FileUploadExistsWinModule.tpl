<div class="title"> </div>
<div class="content">
<h1>{t}File exists{/t}</h1>

{if isset($hasPermission)}
<p>
	{t}Unfortunately the file named{/t} {$file->getFilename()|escape} {t}already exists as an attachment
	to page{/t} {$destinationPage->getTitleOrUnixName()|escape}. {t}Do you want to overwrite it?{/t}
</p>
</div>
<div class="button-bar">
	<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}cancel{/t}</a>
	<a href="javascript:;" onclick="Wikijump.modules.PageUploadModule.listeners.forceOverwrite(event)">{t}overwrite{/t}</a>
</div>
{else}
<p>
	{t}Unfortunately the file named{/t} {$file->getFilename()|escape} {t}already exists as an attachment
	to page{/t} {$destinationPage->getTitleOrUnixName()|escape}. {t}You have no
	permission to overwrite/delete files on this page.{/t}
</p>
</div>
<div class="button-bar">
	<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}close message{/t}</a>
</div>
{/if}
